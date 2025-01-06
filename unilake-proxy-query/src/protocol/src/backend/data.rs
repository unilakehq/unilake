// intent is that here we define the logic to maintain these caches and handle their updates via sse (single sse consumer for all caches) -> use remove_local(key) function on the cache

use casbin::{Cache, DefaultCache};
use redis::cluster::{ClusterClient, ClusterClientBuilder};
use reqwest_eventsource::{Event, EventSource};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use unilake_common::model::{
    AccessPolicyModel, AppInfoModel, EntityModel, GroupModel, IpInfoModel, UserModel,
};
use unilake_common::settings::{
    settings_cache_invalidation_enabled, settings_cache_redis_host, settings_cache_redis_password,
    settings_cache_redis_port, settings_cache_redis_username, settings_server_api_endpoint,
};
use unilake_security::adapter::cached_adapter::{CachedAdapter, CachedPolicyRules};
use unilake_security::caching::layered_cache::{
    BackendProvider, MultiLayeredCache, NoOpCache, RedisBackendProvider,
};
use unilake_security::repository::{CacheContainer, RepoRest};
use unilake_security::HitRule;

pub struct BackendInstance {
    tenant_id: String,
    user_model: Arc<Box<MultiLayeredCache<String, UserModel>>>,
    group_model: Arc<Box<MultiLayeredCache<String, GroupModel>>>,
    entity_model: Arc<Box<MultiLayeredCache<String, EntityModel>>>,
    access_policy_model: Arc<Box<MultiLayeredCache<String, AccessPolicyModel>>>,
    ip_info_model: Arc<Box<MultiLayeredCache<String, IpInfoModel>>>,
    app_info_model: Arc<Box<MultiLayeredCache<String, AppInfoModel>>>,
    policy_cache: Arc<MultiLayeredCache<u64, CachedPolicyRules>>,
    user_rule_hits: RwLock<HashMap<String, (usize, Arc<Box<dyn Cache<u64, (String, HitRule)>>>)>>,
    rest_client: reqwest::Client,
}

impl BackendInstance {
    /// Get the cache container, contains models used for query evaluation (all context)
    pub fn get_cache_container(&self) -> CacheContainer {
        CacheContainer::new(
            self.user_model.clone(),
            self.group_model.clone(),
            self.entity_model.clone(),
            self.access_policy_model.clone(),
        )
    }

    /// Get the user model cache from the multi-layered cache
    pub fn get_ip_info_cache(&self) -> Arc<Box<MultiLayeredCache<String, IpInfoModel>>> {
        self.ip_info_model.clone()
    }

    pub async fn add_user_session(&self, sql_userid: String) {
        if let Some(items) = self.user_rule_hits.write().await.get_mut(&sql_userid) {
            items.0 += 1;
        }
    }

    pub async fn remove_user_session(&self, sql_userid: String) {
        let mut items = self.user_rule_hits.write().await;
        if let Some((counter, _)) = items.get_mut(&sql_userid) {
            *counter -= 1;
            if *counter == 0 {
                items.remove(&sql_userid);
            }
        }
    }

    /// Cached policy hits for improved speed (no need to evaluate the rules again)
    /// Scoped to a user, can be used over multiple sessions from the same user. This cache
    /// is local and not distributed. When the last session of a user is dropped, the cache is also dropped.
    pub async fn get_user_hit_rules(
        &self,
        sql_userid: String,
    ) -> Arc<Box<dyn Cache<u64, (String, HitRule)>>> {
        if let Some(cached_hits) = self.user_rule_hits.read().await.get(&sql_userid) {
            return cached_hits.1.clone();
        }
        let mut items = self.user_rule_hits.write().await;
        let cache: Arc<Box<dyn Cache<u64, (String, HitRule)>>> =
            Arc::new(Box::new(DefaultCache::new(100)));
        items.insert(sql_userid, (1, cache.clone()));
        cache
    }

    /// Get the app info cache from the multi-layered cache
    pub fn get_app_info_cache(&self) -> Arc<Box<MultiLayeredCache<String, AppInfoModel>>> {
        self.app_info_model.clone()
    }

    /// Adapter is used for loading policy rules, in this case from a multi-layered cache
    pub fn get_cached_adapter(&self) -> CachedAdapter {
        CachedAdapter::new(self.policy_cache.clone())
    }

    /// Get the current active policy id from the multi-layered cache
    pub async fn get_active_policy_id(&self) -> Option<u64> {
        // id 0 is always used as a placeholder for the current active policy id
        match self.policy_cache.get(&0u64).await {
            Some(CachedPolicyRules::PolicyId(id)) => Some(id),
            _ => {
                tracing::error!("No rules cache found for tenant {}", self.tenant_id);
                None
            }
        }
    }

    /// Clears all local data related to the specified tenant
    pub fn clear_local_data(&self) {
        self.user_model.clear();
        self.group_model.clear();
        self.entity_model.clear();
        self.access_policy_model.clear();
        self.ip_info_model.clear();
        self.app_info_model.clear();
        self.policy_cache.clear();
    }

    pub fn get_rest_client(&self) -> reqwest::Client {
        self.rest_client.clone()
    }
}

#[derive(Clone, Debug, Deserialize)]
struct SseEventDto {
    #[serde(rename = "tenantId")]
    tenant_id: String,
    #[serde(rename = "invalidationRequest")]
    invalidation_request: Option<SseInvalidateRequestDto>,
}

#[derive(Clone, Debug, Deserialize)]
struct SseInvalidateRequestDto {
    #[serde(rename = "cacheType")]
    cache_type: String,
    key: String,
}

pub struct BackendHandler {
    instances: RwLock<HashMap<String, Arc<BackendInstance>>>,
    redis_client: Option<Arc<ClusterClient>>,
    backend_running: RwLock<bool>,
    rest_client: Option<reqwest::Client>,
}

impl BackendHandler {
    pub fn new() -> Self {
        let redis_client = Self::get_redis_client();
        BackendHandler {
            redis_client,
            instances: RwLock::new(HashMap::new()),
            backend_running: RwLock::new(false),
            rest_client: Some(reqwest::Client::new()),
        }
    }

    fn get_redis_client() -> Option<Arc<ClusterClient>> {
        tracing::info!("Initializing Redis client");
        match settings_cache_redis_host() {
            None => {
                tracing::warn!("Redis cache is disabled. Falling back to local caching only.");
                None
            }
            Some(v) => {
                let connections = v
                    .split(',')
                    .map(|s| format!("{}:{}", s.trim(), settings_cache_redis_port()))
                    .collect::<Vec<String>>();
                match ClusterClientBuilder::new(connections)
                    .username(settings_cache_redis_username().unwrap_or("".to_owned()))
                    .password(settings_cache_redis_password().unwrap_or("".to_owned()))
                    .build()
                {
                    Ok(c) => {
                        tracing::info!("Successfully created Redis client");
                        Some(Arc::new(c))
                    }
                    Err(e) => {
                        tracing::error!("Error creating redis client, falling back to local caching only. Error message: {}", e);
                        None
                    }
                }
            }
        }
    }

    pub fn get_rest_client(&self) -> reqwest::Client {
        self.rest_client
            .clone()
            .unwrap_or_else(|| panic!("No REST client configured"))
    }

    /// Returns either a distributed cache (redis) or a no-op cache (local)
    fn get_distributed_cache<K, V>(
        &self,
        tenant_id: String,
        backend_type: String,
    ) -> Box<dyn BackendProvider<K, V>>
    where
        K: Send + Hash + Clone + Sync + Eq,
        V: Send + Serialize + DeserializeOwned + Clone + Send + Sync,
    {
        match self.redis_client.clone() {
            None => Box::new(NoOpCache::new(tenant_id, backend_type.to_owned())),
            Some(redis_client) => Box::from(RedisBackendProvider::new(
                redis_client,
                tenant_id,
                backend_type,
            )),
        }
    }

    /// Add a new tenant to the backend handler
    async fn add_tenant(&self, tenant_id: String) -> Arc<BackendInstance> {
        let local_cap = 100;
        let backend_instance = BackendInstance {
            tenant_id: tenant_id.to_owned(),
            user_rule_hits: RwLock::new(HashMap::new()),
            user_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                self.get_distributed_cache(tenant_id.to_owned(), "user_model".to_owned()),
                Box::from(RepoRest::new(tenant_id.to_owned(), self.get_rest_client())),
            ))),
            group_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                self.get_distributed_cache(tenant_id.to_owned(), "group_model".to_owned()),
                Box::from(RepoRest::new(tenant_id.to_owned(), self.get_rest_client())),
            ))),
            entity_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                self.get_distributed_cache(tenant_id.to_owned(), "entity_model".to_owned()),
                Box::from(RepoRest::new(tenant_id.to_owned(), self.get_rest_client())),
            ))),
            access_policy_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                self.get_distributed_cache(tenant_id.to_owned(), "access_policy_model".to_owned()),
                Box::from(RepoRest::new(tenant_id.to_owned(), self.get_rest_client())),
            ))),
            app_info_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                self.get_distributed_cache(tenant_id.to_owned(), "app_info_model".to_owned()),
                Box::from(RepoRest::new(tenant_id.to_owned(), self.get_rest_client())),
            ))),
            ip_info_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                self.get_distributed_cache(tenant_id.to_owned(), "ip_info_model".to_owned()),
                Box::from(RepoRest::new(tenant_id.to_owned(), self.get_rest_client())),
            ))),
            policy_cache: Arc::new(MultiLayeredCache::new(
                local_cap,
                self.get_distributed_cache(tenant_id.to_owned(), "policy_cache".to_owned()),
                Box::from(RepoRest::new(tenant_id.to_owned(), self.get_rest_client())),
            )),
            rest_client: self.get_rest_client(),
        };

        let backend_instance = Arc::new(backend_instance);
        self.instances
            .write()
            .await
            .insert(tenant_id, Arc::clone(&backend_instance));

        backend_instance
    }

    /// Get the backend instance for a specific tenant.
    /// will add the tenant if it doesn't exist.
    pub async fn get_backend_instance(&self, tenant_id: String) -> Arc<BackendInstance> {
        if let Some(instance) = self.instances.read().await.get(&tenant_id) {
            return instance.clone();
        }
        self.add_tenant(tenant_id).await
    }

    /// Clears all instances for a specific tenant.
    /// To be called when a tenant has no connections left on this proxy.
    /// Tenant can get the data from the distribute cache on reconnect. In case this instance is used in
    /// a single tenant environment, you can disable the cache invalidation feature for improved performance.
    pub async fn clear_tenant_instances(&self, tenant_id: &str, forced: bool) {
        if settings_cache_invalidation_enabled() || forced {
            tracing::warn!(
                "Invalidating cache for tenant {}, forced: {}",
                tenant_id,
                forced
            );
            self.get_backend_instance(tenant_id.to_string())
                .await
                .clear_local_data();
        }
    }

    async fn on_sse_event(&self, update: SseEventDto) {
        tracing::info!("Received SSE action: {:?}", update);
        let instance = self.get_backend_instance(update.tenant_id.clone()).await;
        if let Some(invalidation_reques) = update.invalidation_request {
            match invalidation_reques.cache_type.as_str() {
                "user" => {
                    instance
                        .user_model
                        .remove_local(&invalidation_reques.key)
                        .await
                }
                "group" => {
                    instance
                        .group_model
                        .remove_local(&invalidation_reques.key)
                        .await
                }
                "entity" => {
                    instance
                        .entity_model
                        .remove_local(&invalidation_reques.key)
                        .await
                }
                "access_policy" => {
                    instance
                        .access_policy_model
                        .remove_local(&invalidation_reques.key)
                        .await
                }
                "ip_info" => {
                    instance
                        .ip_info_model
                        .remove_local(&invalidation_reques.key)
                        .await
                }
                "app_info" => {
                    instance
                        .app_info_model
                        .remove_local(&invalidation_reques.key)
                        .await
                }
                "policy" => instance.policy_cache.clear(),
                "all" => {
                    self.clear_tenant_instances(&update.tenant_id, true).await;
                }
                _ => {
                    tracing::warn!("Unknown cache type: {}", invalidation_reques.cache_type);
                }
            }
        }
    }

    /// Clears all local data for all tenants.
    async fn clear_all_instances(&self) {
        let instances = self.instances.write().await;
        for (tenant_id, _) in instances.iter() {
            self.clear_tenant_instances(tenant_id, true).await;
        }
    }

    /// Starts an SSE Consumer for all tenants to check for SSE updates and invalidate local caches.
    /// Depending on your permission, this sse consumer will either receive all tenants data or just your own tenant
    pub async fn start_sse_consumer(backend_handler: Arc<Self>) {
        if *backend_handler.backend_running.read().await {
            panic!("SSE consumer already running, only one instance is allowed.");
        }

        tokio::spawn(async move {
            let mut backoff = 1;
            *backend_handler.backend_running.write().await = true;

            // todo: add hmac based authentication
            let endpoint = format!(
                "{}/security/proxy/event-stream",
                settings_server_api_endpoint()
            );
            tracing::info!("Starting SSE consumer at {}", endpoint);
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                let mut es = EventSource::get(endpoint.clone());
                while let Some(event) = es.next().await {
                    match event {
                        Ok(Event::Open) => {
                            tracing::info!(
                                "SSE consumer connected, clearing caches on initial run."
                            );
                            backend_handler.clear_all_instances().await;
                        }
                        Ok(Event::Message(message)) => {
                            tracing::info!("Received SSE update: {:?}", message);
                            match message.event.as_str() {
                                "update" => {
                                    match serde_json::from_str::<SseEventDto>(&message.data) {
                                        Ok(event) => backend_handler.on_sse_event(event).await,
                                        Err(e) => {
                                            tracing::error!("Error parsing SSE update: {}", e);
                                        }
                                    }
                                }
                                _ => tracing::warn!("Unknown SSE event: {}", message.event),
                            }
                        }
                        Err(err) => {
                            backoff *= 2;
                            backoff = std::cmp::min(backoff, 30);
                            tracing::error!(
                                "Error in SSE consumer: {}. Reconnecting in {} seconds",
                                err,
                                backoff
                            );
                        }
                    }
                }
            }
        });
    }
}
