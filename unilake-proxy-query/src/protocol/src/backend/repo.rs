// intent is that here we define the logic to maintain these caches and handle their updates via sse (single sse consumer for all caches) -> use remove_local(key) function on the cache

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use unilake_common::model::{
    AccessPolicyModel, AppInfoModel, EntityModel, GroupModel, IpInfoModel, UserModel,
};
use unilake_common::settings::settings_cache_invalidation_enabled;
use unilake_security::adapter::cached_adapter::{CacheEntity, CachedAdapter};
use unilake_security::caching::layered_cache::{MultiLayeredCache, RedisBackendProvider};
use unilake_security::repository::{CacheContainer, RepoRest};

struct BackendInstance {
    tenant_id: String,
    user_model: Arc<Box<MultiLayeredCache<String, UserModel>>>,
    group_model: Arc<Box<MultiLayeredCache<String, GroupModel>>>,
    entity_model: Arc<Box<MultiLayeredCache<String, EntityModel>>>,
    access_policy_model: Arc<Box<MultiLayeredCache<String, AccessPolicyModel>>>,
    ip_info_model: Arc<Box<MultiLayeredCache<String, IpInfoModel>>>,
    app_info_model: Arc<Box<MultiLayeredCache<String, AppInfoModel>>>,
    policy_cache: Arc<MultiLayeredCache<u64, CacheEntity>>,
}

impl BackendInstance {
    fn get_cache_container(&self) -> CacheContainer {
        CacheContainer::new(
            self.user_model.clone(),
            self.group_model.clone(),
            self.entity_model.clone(),
            self.access_policy_model.clone(),
        )
    }

    pub fn get_ip_info_cache(&self) -> Arc<Box<MultiLayeredCache<String, IpInfoModel>>> {
        self.ip_info_model.clone()
    }

    pub fn get_app_info_cache(&self) -> Arc<Box<MultiLayeredCache<String, AppInfoModel>>> {
        self.app_info_model.clone()
    }

    pub fn get_policy_adapter(&self) -> CachedAdapter {
        CachedAdapter::new(self.policy_cache.clone())
    }
}

#[derive(Clone, Debug, Deserialize)]
struct SseInvalidateRequestDto {
    tenant_id: String,
    cache_type: String,
    key: String,
}

struct BackendHandler {
    instances: RwLock<HashMap<String, Arc<BackendInstance>>>,
    redis_client: Arc<redis::Client>,
    repo_backend: Box<RepoRest>,
}

impl BackendHandler {
    pub async fn add_tenant_workspace(&self, tenant_id: &str, workspace_id: &str) {
        let local_cap = 100;
        let backend_instance = BackendInstance {
            tenant_id: tenant_id.to_owned(),
            user_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                Box::from(RedisBackendProvider::new(
                    self.redis_client.clone(),
                    tenant_id,
                    "ip_info",
                )),
                Box::from(RepoRest::new(tenant_id, workspace_id)),
            ))),
            group_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                Box::from(RedisBackendProvider::new(
                    self.redis_client.clone(),
                    tenant_id,
                    "ip_info",
                )),
                Box::from(RepoRest::new(tenant_id, workspace_id)),
            ))),
            entity_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                Box::from(RedisBackendProvider::new(
                    self.redis_client.clone(),
                    tenant_id,
                    "ip_info",
                )),
                Box::from(RepoRest::new(tenant_id, workspace_id)),
            ))),
            access_policy_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                Box::from(RedisBackendProvider::new(
                    self.redis_client.clone(),
                    tenant_id,
                    "ip_info",
                )),
                Box::from(RepoRest::new(tenant_id, workspace_id)),
            ))),
            app_info_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                Box::from(RedisBackendProvider::new(
                    self.redis_client.clone(),
                    tenant_id,
                    "ip_info",
                )),
                Box::from(RepoRest::new(tenant_id, workspace_id)),
            ))),
            ip_info_model: Arc::new(Box::new(MultiLayeredCache::new(
                local_cap,
                Box::from(RedisBackendProvider::new(
                    self.redis_client.clone(),
                    tenant_id,
                    "ip_info",
                )),
                Box::from(RepoRest::new(tenant_id, workspace_id)),
            ))),
        };

        self.instances
            .write()
            .await
            .insert(tenant_id.to_string(), Arc::new(backend_instance));
    }

    /// Get the backend instance for a specific tenant.
    pub async fn get_backend_instance(&self, tenant_id: &str) -> Option<Arc<BackendInstance>> {
        self.instances
            .read()
            .await
            .get(&tenant_id.to_string())
            .cloned()
    }

    /// Clears all instances for a specific tenant.
    /// To be called when a tenant has no connections left on this proxy.
    /// Tenant can get the data from the distribute cache on reconnect. In case this instance is used in
    /// a single tenant environment, you can disable the cache invalidation feature for improved performance.
    pub async fn clear_tenant_instances(&self, tenant_id: &str) {
        if settings_cache_invalidation_enabled() {
            self.instances.write().await.remove(&tenant_id.to_string());
        }
    }

    // todo: handle updates from sse
    // todo: handle clearing instances when running out of memory?
    async fn on_sse_update(&mut self, update: SseInvalidateRequestDto) {
        tracing::info!("Received SSE update: {:?}", update);

        if let Some(instance) = self.get_backend_instance(&update.tenant_id).await {
            match update.cache_type.as_str() {
                "user" => instance.user_model.remove_local(&update.key).await,
                "group" => instance.group_model.remove_local(&update.key).await,
                "entity" => instance.entity_model.remove_local(&update.key).await,
                "access_policy" => instance.access_policy_model.remove_local(&update.key).await,
                "ip_info" => instance.ip_info_model.remove_local(&update.key).await,
                "app_info" => instance.app_info_model.remove_local(&update.key).await,
                "policy" => instance.policy_cache.clear(),
                _ => {
                    tracing::warn!("Unknown cache type: {}", update.cache_type);
                }
            }
        }
    }
}
