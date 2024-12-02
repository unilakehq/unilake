// Note: use backon for resilience
use crate::caching::layered_cache::{BackendProvider, MultiLayeredCache};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::hash::Hash;
use std::sync::Arc;
use unilake_common::model::{
    AccessPolicyModel, DataAccessRequest, DataAccessRequestResponse, EntityModel, GroupModel,
    UserModel,
};
use unilake_common::settings::global_config;

pub struct CacheContainer {
    pub user_model: Arc<Box<MultiLayeredCache<String, UserModel>>>,
    pub group_model: Arc<Box<MultiLayeredCache<String, GroupModel>>>,
    pub entity_model: Arc<Box<MultiLayeredCache<String, EntityModel>>>,
    pub access_policy_model: Arc<Box<MultiLayeredCache<String, AccessPolicyModel>>>,
}

impl CacheContainer {
    pub fn new(
        user_model_cache: Arc<Box<MultiLayeredCache<String, UserModel>>>,
        group_model_cache: Arc<Box<MultiLayeredCache<String, GroupModel>>>,
        entity_model_cache: Arc<Box<MultiLayeredCache<String, EntityModel>>>,
        policy_model_cache: Arc<Box<MultiLayeredCache<String, AccessPolicyModel>>>,
    ) -> Self {
        Self {
            user_model: user_model_cache,
            group_model: group_model_cache,
            entity_model: entity_model_cache,
            access_policy_model: policy_model_cache,
        }
    }
}

#[async_trait]
impl<K, T> BackendProvider<K, AccessPolicyModel> for T
where
    K: ToString + Send + Sync + Hash,
    T: RepoBackend,
{
    async fn get(&self, key: &K) -> Result<Option<AccessPolicyModel>, String> {
        self.get_access_policy_model(key.to_string()).await
    }

    async fn set(&self, _: &K, _: &AccessPolicyModel) -> Result<(), String> {
        unreachable!("Set is not implemented for RepoRest")
    }

    async fn has(&self, key: &K) -> Result<bool, String> {
        self.get(key)
            .await
            .map(|opt: Option<AccessPolicyModel>| opt.is_some())
    }

    async fn evict(&self, _: &K) -> Result<(), String> {
        Ok(())
    }

    fn generate_key(&self, _: &K) -> String {
        unreachable!("Key generation is not implemented for RepoBackend")
    }
}

#[async_trait]
impl<K, T> BackendProvider<K, EntityModel> for T
where
    K: ToString + Send + Sync + Hash,
    T: RepoBackend,
{
    async fn get(&self, name: &K) -> Result<Option<EntityModel>, String> {
        self.get_entity_model(name.to_string()).await
    }

    async fn set(&self, _: &K, _: &EntityModel) -> Result<(), String> {
        unreachable!("Set is not implemented for RepoRest")
    }

    async fn has(&self, key: &K) -> Result<bool, String> {
        self.get(key)
            .await
            .map(|opt: Option<EntityModel>| opt.is_some())
    }

    async fn evict(&self, _: &K) -> Result<(), String> {
        Ok(())
    }

    fn generate_key(&self, _: &K) -> String {
        unreachable!("Key generation is not implemented for RepoBackend")
    }
}

#[async_trait]
impl<K, T> BackendProvider<K, UserModel> for T
where
    K: ToString + Send + Sync + Hash,
    T: RepoBackend,
{
    async fn get(&self, key: &K) -> Result<Option<UserModel>, String> {
        self.get_user_model(key.to_string()).await
    }

    async fn set(&self, _: &K, _: &UserModel) -> Result<(), String> {
        unreachable!("Set is not implemented for RepoRest")
    }

    async fn has(&self, key: &K) -> Result<bool, String> {
        self.get(key)
            .await
            .map(|opt: Option<UserModel>| opt.is_some())
    }

    async fn evict(&self, _: &K) -> Result<(), String> {
        Ok(())
    }

    fn generate_key(&self, _: &K) -> String {
        unreachable!("Key generation is not implemented for RepoBackend")
    }
}

#[async_trait]
impl<K, T> BackendProvider<K, GroupModel> for T
where
    K: ToString + Send + Sync + Hash,
    T: RepoBackend,
{
    async fn get(&self, key: &K) -> Result<Option<GroupModel>, String> {
        self.get_group_model(key.to_string()).await
    }

    async fn set(&self, _: &K, _: &GroupModel) -> Result<(), String> {
        unreachable!("Set is not implemented for RepoRest")
    }

    async fn has(&self, key: &K) -> Result<bool, String> {
        self.get(key)
            .await
            .map(|opt: Option<GroupModel>| opt.is_some())
    }

    async fn evict(&self, _: &K) -> Result<(), String> {
        Ok(())
    }

    fn generate_key(&self, _: &K) -> String {
        unreachable!("Key generation is not implemented for RepoBackend")
    }
}

#[async_trait]
pub trait RepoBackend: Send + Sync {
    async fn get_entity_model(&self, name: String) -> Result<Option<EntityModel>, String>;
    async fn get_access_policy_model(
        &self,
        id: String,
    ) -> Result<Option<AccessPolicyModel>, String>;
    async fn get_user_model(&self, id: String) -> Result<Option<UserModel>, String>;
    async fn get_group_model(&self, id: String) -> Result<Option<GroupModel>, String>;
    async fn generate_access_request(
        &self,
        user_id: String,
        security_policy_id: String,
    ) -> Result<DataAccessRequestResponse, String>;
    async fn get_access_by_action(
        &self,
        catalog: String,
        schema: String,
        entity: Option<String>,
        action: String,
    ) -> Result<bool, String>;
}

// todo: add auth (service account)
pub struct RepoRest {
    tenant_id: String,
    workspace_id: String,
    api_endpoint: String,
    client: reqwest::Client,
}

impl RepoRest {
    #[allow(dead_code)]
    pub fn new(tenant_id: String, workspace_id: String) -> Self {
        let api_endpoint = global_config().get::<String>("api_endpoint").unwrap();
        RepoRest {
            tenant_id,
            workspace_id,
            client: reqwest::Client::new(),
            api_endpoint,
        }
    }

    async fn get_request<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let url = self.get_path(path);
        let response = match self.client.get(url.clone()).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("GET Request failed: {}", e);
                return Err(format!("Failed to send GET request: {}", url));
            }
        };
        match response.json::<T>().await {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::error!("Failed to parse JSON response: {}", e);
                Err(format!("Failed to parse JSON response: {}", url))
            }
        }
    }

    async fn post_request<I: Serialize, O: DeserializeOwned>(
        &self,
        path: &str,
        data: I,
    ) -> Result<O, String> {
        let url = self.get_path(path);
        let response = match self.client.post(url.clone()).json(&data).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("POST Request failed: {}", e);
                return Err(format!("Failed to send POST request: {}", url));
            }
        };

        match response.json::<O>().await {
            Ok(r) => Ok(r),
            Err(e) => {
                tracing::error!("Failed to parse JSON response: {}", e);
                Err(format!("Failed to parse JSON response: {}", url))
            }
        }
    }

    fn get_path(&self, path: &str) -> String {
        format!(
            "{}/{}/{}/{}",
            self.api_endpoint, self.tenant_id, self.workspace_id, path
        )
    }
}

#[async_trait]
impl RepoBackend for RepoRest {
    async fn get_entity_model(&self, name: String) -> Result<Option<EntityModel>, String> {
        self.get_request::<Option<EntityModel>>(
            format!("security/proxy/entity-models/{}", name).as_str(),
        )
        .await
    }

    async fn get_access_policy_model(
        &self,
        id: String,
    ) -> Result<Option<AccessPolicyModel>, String> {
        self.get_request::<Option<AccessPolicyModel>>(
            format!("security/proxy/access-policy-models/{}", id).as_str(),
        )
        .await
    }

    async fn get_user_model(&self, id: String) -> Result<Option<UserModel>, String> {
        self.get_request::<Option<UserModel>>(format!("security/proxy/user-models/{}", id).as_str())
            .await
    }

    async fn get_group_model(&self, id: String) -> Result<Option<GroupModel>, String> {
        self.get_request::<Option<GroupModel>>(
            format!("security/proxy/group-models/{}", id).as_str(),
        )
        .await
    }

    async fn generate_access_request(
        &self,
        user_id: String,
        security_policy_id: String,
    ) -> Result<DataAccessRequestResponse, String> {
        self.post_request(
            "security/access-requests/generate",
            DataAccessRequest {
                user_id,
                security_policy_id,
            },
        )
        .await
    }

    async fn get_access_by_action(
        &self,
        catalog: String,
        schema: String,
        entity: Option<String>,
        action: String,
    ) -> Result<bool, String> {
        let mut url = format!("determine/path/{}/{}", catalog, schema);
        if let Some(entity) = entity {
            url = format!("{}/{}", url, entity);
        }
        url = format!("{}/{}", url, action);
        self.get_request(url.as_str()).await
    }
}
