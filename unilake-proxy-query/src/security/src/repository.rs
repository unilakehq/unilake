// TODO: implementation for acquiring policy information (PIP)
// Note: use backon for resilience
use crate::caching::layered_cache::{BackendProvider, MultiLayeredCache};
use async_trait::async_trait;
use std::hash::Hash;
use std::sync::Arc;
use unilake_common::model::{AccessPolicyModel, EntityModel, GroupModel, UserModel};

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
impl<K> BackendProvider<K, AccessPolicyModel> for RepoRest
where
    K: Send + Hash + Clone + Eq + Sync + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<AccessPolicyModel>, String> {
        todo!()
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
impl<K> BackendProvider<K, EntityModel> for RepoRest
where
    K: Send + Hash + Clone + Eq + Sync + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<EntityModel>, String> {
        todo!()
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
impl<K> BackendProvider<K, UserModel> for RepoRest
where
    K: Send + Hash + Clone + Eq + Sync + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<UserModel>, String> {
        todo!()
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
impl<K> BackendProvider<K, GroupModel> for RepoRest
where
    K: Send + Hash + Clone + Eq + Sync + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<GroupModel>, String> {
        todo!()
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

pub struct RepoRest {
    api_endpoint: String,
}
