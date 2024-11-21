use async_trait::async_trait;
use moka::future::Cache as MokaCache;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisResult};
use rslock::{Lock, LockManager};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

fn get_key_hash<H>(key: H) -> u64
where
    H: Hash,
{
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

pub struct MultiLayeredCache<K, V> {
    local_cache: MokaCache<K, V>,
    distributed_cache: Box<dyn BackendProvider<K, V>>,
    backend_repo: Box<dyn BackendProvider<K, V>>,
    lock_manager: Option<LockManager>,
}

impl<K, V> MultiLayeredCache<K, V>
where
    K: Send + Hash + Clone + Eq + Sync + 'static,
    V: Send + Serialize + DeserializeOwned + Sync + Clone + 'static,
{
    pub fn new(
        local_cap: u64,
        distributed_cache: Box<dyn BackendProvider<K, V>>,
        backend_repo: Box<dyn BackendProvider<K, V>>,
    ) -> MultiLayeredCache<K, V> {
        MultiLayeredCache {
            local_cache: MokaCache::<K, V>::builder()
                .weigher(|_key, value| -> u32 { size_of_val(&*value) as u32 })
                .max_capacity(local_cap * 1024 * 1024)
                .time_to_live(Duration::from_secs(15 * 60)) // 15 minutes
                .build(),
            distributed_cache,
            backend_repo,
            lock_manager: None,
        }
    }

    pub fn set_lock_manager(&mut self, uris: Vec<String>) {
        let manager = LockManager::new(uris);
        self.lock_manager = Some(manager);
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        match self.local_cache.get(key).await {
            Some(v) => Some(v),
            None => {
                // get from backend
                if let Ok(Some(v)) = self.distributed_cache.get(key).await {
                    self.local_cache.insert(key.clone(), v.clone()).await;
                    return Some(v);
                }
                // get from repo
                match self.get_from_repo(key).await {
                    Ok(v) => return v,
                    Err(e) => tracing::error!("Error getting data from repo: {}", e),
                }
                None
            }
        }
    }

    async fn get_lock(&self, key: &K) -> Option<Lock> {
        // acquire lock
        if let Some(ref lm) = self.lock_manager {
            let key = self.distributed_cache.generate_key(key);
            let key_bytes = key.as_bytes();
            tracing::info!("Trying to acquire lock for key: {}.", key);
            loop {
                if let Ok(lock) = lm.lock(key_bytes, Duration::from_millis(3000)).await {
                    tracing::info!("Acquired lock for key: {}.", key);
                    return Some(lock);
                }
            }
        }
        None
    }

    async fn release_lock(&self, key: &K, lock: Option<Lock>) {
        if let Some(lock) = lock {
            let key = self.distributed_cache.generate_key(key);
            tracing::info!("Released lock for key: {}.", key);

            // unwrap since you cannot have a lock without this lock_manager
            self.lock_manager.as_ref().unwrap().unlock(&lock).await;
        }
    }

    async fn get_from_repo(&self, key: &K) -> Result<Option<V>, String> {
        // acquire lock if enabled
        let lock: Option<Lock> = self.get_lock(key).await;

        // try to get from cache (just in case data has been refreshed)
        if self.has(key).await {
            self.release_lock(key, lock).await;
            return Ok(self.distributed_cache.get(key).await?);
        }

        // get data from repo
        let result = self.backend_repo.get(key).await?;

        // set data in local and distributed caches
        if let Some(ref v) = result {
            self.set(key.clone(), v.clone()).await;
            self.local_cache.insert(key.clone(), v.clone()).await;
        }

        // release lock, if applicable
        self.release_lock(key, lock).await;
        Ok(result)
    }

    pub async fn has(&self, k: &K) -> bool {
        let found = self.local_cache.contains_key(k);
        if !found {
            return self.distributed_cache.has(k).await.unwrap_or(false);
        }
        found
    }

    pub async fn set(&self, key: K, value: V) {
        let _ = self.distributed_cache.set(&key, &value).await;
        self.local_cache.insert(key, value).await;
    }

    pub fn clear(&self) {
        self.local_cache.invalidate_all();
    }
}

#[async_trait]
pub trait BackendProvider<K, V>: Send + Sync
where
    K: Send + Hash,
    V: Send + Serialize + DeserializeOwned,
{
    async fn get(&self, key: &K) -> Result<Option<V>, String>;
    async fn set(&self, key: &K, value: &V) -> Result<(), String>;
    async fn has(&self, key: &K) -> Result<bool, String>;
    async fn evict(&self, key: &K) -> Result<(), String>;
    fn generate_key(&self, key: &K) -> String;
}

pub struct RedisBackendProvider {
    // Implement Redis connection logic
    client: redis::Client,
    tenant_id: String,
    /// Can be either Policy, GroupModel, UserModel, EntityModel
    backend_type: String,
}

impl RedisBackendProvider {
    pub fn new(host: &str, port: u16, tenant_id: &str, backend_type: &str) -> RedisBackendProvider {
        RedisBackendProvider {
            client: redis::Client::open(format!("redis://{}:{}", host, port)).unwrap(),
            tenant_id: tenant_id.to_owned(),
            backend_type: backend_type.to_owned(),
        }
    }

    async fn get_connection(&self) -> Result<MultiplexedConnection, String> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| "Failed to connect to Redis".to_string())
    }

    fn generate_key<H>(&self, key: &H) -> String
    where
        H: Hash,
    {
        format!(
            "{}:{}:{}",
            self.tenant_id,
            self.backend_type,
            get_key_hash(key)
        )
    }
}

#[async_trait]
impl<K, V> BackendProvider<K, V> for RedisBackendProvider
where
    K: Send + Hash + Sync,
    V: Send + Serialize + DeserializeOwned + Sync,
{
    async fn get(&self, key: &K) -> Result<Option<V>, String> {
        let mut conn = self.get_connection().await?;
        let key_str = self.generate_key(key);
        let found: RedisResult<String> = conn.get(&key_str).await;
        match found {
            Ok(t) => Ok(Some(serde_json::from_str(&t).unwrap())),
            Err(_) => Ok(None),
        }
    }

    async fn set(&self, key: &K, value: &V) -> Result<(), String> {
        let mut conn = self.get_connection().await?;
        let key_str = self.generate_key(key);
        conn.set(&key_str, serde_json::to_string(value).unwrap())
            .await
            .map_err(|_| "Failed to set".to_string())?;
        conn.expire(&key_str, 60 * 60)
            .await
            .map_err(|_| "Failed to set expire".to_string()) // 1 hour
    }

    async fn has(&self, key: &K) -> Result<bool, String> {
        let mut conn = self.get_connection().await?;
        let key_str = self.generate_key(key);
        conn.exists(key_str)
            .await
            .map_err(|_| "Failed to check existence".to_string())
    }

    async fn evict(&self, key: &K) -> Result<(), String> {
        let mut conn = self.get_connection().await?;
        let key_str = self.generate_key(key);
        conn.del(key_str)
            .await
            .map_err(|_| "Failed to evict".to_string())
    }

    fn generate_key(&self, key: &K) -> String {
        self.generate_key(key)
    }
}

mod tests {
    use super::*;
    struct TestBackendProvider;

    #[async_trait]
    impl BackendProvider<Vec<String>, bool> for TestBackendProvider {
        async fn get(&self, _: &Vec<String>) -> Result<Option<bool>, String> {
            Ok(None)
        }

        async fn set(&self, _: &Vec<String>, _: &bool) -> Result<(), String> {
            Ok(())
        }

        async fn has(&self, _: &Vec<String>) -> Result<bool, String> {
            Ok(false)
        }

        async fn evict(&self, _: &Vec<String>) -> Result<(), String> {
            Ok(())
        }

        fn generate_key(&self, key: &Vec<String>) -> String {
            format!("{:?}", key)
        }
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let backend = Box::new(TestBackendProvider);
        let repo = Box::new(TestBackendProvider);
        let cache = MultiLayeredCache::new(1, backend, repo);

        cache
            .set(
                vec![
                    "alice".to_string(),
                    "/data1".to_string(),
                    "read".to_string(),
                ],
                true,
            )
            .await;
        let result = cache
            .get(&vec![
                "alice".to_string(),
                "/data1".to_string(),
                "read".to_string(),
            ])
            .await;
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_has_and_clear() {
        // let channel = channel::<(String, Option<bool>)>();
        // let cache = MultiLayeredCache::new(1, TestBackendProvider, channel.0);
        //
        // cache.set(vec!["alice", "/data1", "read"], false);
        // assert!(cache.has(&vec!["alice", "/data1", "read"]));
        // cache.clear();
        // assert!(!cache.has(&vec!["alice", "/data1", "read"]));
    }
}
