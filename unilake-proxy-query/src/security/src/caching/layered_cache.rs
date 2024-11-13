use async_trait::async_trait;
use moka::future::Cache as MokaCache;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisResult};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::hash::{DefaultHasher, Hash, Hasher};

fn get_key_hash<H>(key: H) -> u64
where
    H: Hash,
{
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

pub struct MultiLayeredCache<K, V>
where
    K: Send + Hash + Clone + Eq + Sync + 'static,
    V: Send + Serialize + DeserializeOwned + Sync + Clone + 'static,
{
    cache: MokaCache<K, V>,
    backend: Box<dyn BackendProvider<K, V>>,
}

impl<K, V> MultiLayeredCache<K, V>
where
    K: Send + Hash + Clone + Eq + Sync + 'static,
    V: Send + Serialize + DeserializeOwned + Sync + Clone + 'static,
{
    pub fn new(cap: u64, backend: Box<dyn BackendProvider<K, V>>) -> MultiLayeredCache<K, V> {
        MultiLayeredCache {
            cache: MokaCache::<K, V>::builder()
                .weigher(|_key, value| -> u32 { size_of_val(&*value) as u32 })
                .max_capacity(cap * 1024 * 1024)
                .time_to_live(std::time::Duration::from_secs(15 * 60)) // 15 minutes
                .build(),
            backend,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        match self.cache.get(key).await {
            Some(v) => Some(v),
            None => {
                if let Ok(Some(v)) = self.backend.get(key).await {
                    self.cache.insert(key.clone(), v.clone()).await;
                    return Some(v);
                }
                None
            }
        }
    }

    pub async fn has(&self, k: &K) -> bool {
        let found = self.cache.contains_key(k);
        if !found {
            return self.backend.has(k).await.unwrap_or(false);
        }
        found
    }

    pub async fn set(&self, key: K, value: V) {
        let _ = self.backend.set(&key, &value).await;
        self.cache.insert(key, value).await;
    }

    pub fn clear(&self) {
        self.cache.invalidate_all();
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
    /// Can be either Policy, GroupModel, UserModel, ObjectModel, EntityModel
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
        let cache = MultiLayeredCache::new(1, backend);

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
