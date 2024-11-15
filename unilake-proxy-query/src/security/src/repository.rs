// TODO: implementation for acquiring policy information (PIP)
// Note: use backon for resilience
use crate::caching::layered_cache::MultiLayeredCache;
use crate::handler::SecurityHandlerResult;
use std::sync::Arc;
use unilake_common::model::{EntityModel, GroupModel, ObjectModel, UserModel};

pub struct CacheContainer {
    user_model_cache: Arc<Box<MultiLayeredCache<String, UserModel>>>,
    group_model_cache: Arc<Box<MultiLayeredCache<String, GroupModel>>>,
    entity_model_cache: Arc<Box<MultiLayeredCache<String, EntityModel>>>,
    object_model_cache: Arc<Box<MultiLayeredCache<String, ObjectModel>>>,
}

impl CacheContainer {
    pub fn new(
        user_model_cache: Arc<Box<MultiLayeredCache<String, UserModel>>>,
        group_model_cache: Arc<Box<MultiLayeredCache<String, GroupModel>>>,
        object_model_cache: Arc<Box<MultiLayeredCache<String, ObjectModel>>>,
        entity_model_cache: Arc<Box<MultiLayeredCache<String, EntityModel>>>,
    ) -> Self {
        Self {
            user_model_cache,
            group_model_cache,
            object_model_cache,
            entity_model_cache,
        }
    }
}
pub struct RepoBackend {
    cache_container: CacheContainer,
    repo_rest: RepoRest,
}

impl RepoBackend {
    pub async fn get_object_model(
        &self,
        object_key: &String,
    ) -> Result<ObjectModel, SecurityHandlerResult> {
        if let Some(found) = self
            .cache_container
            .object_model_cache
            .get(object_key)
            .await
        {
            return Ok(found);
        }

        let fetched_model = self
            .repo_rest
            .fetch_objectmodel(object_key.as_str())
            .await
            .map_err(|e| SecurityHandlerResult::RepositoryError(e))?;

        self.cache_container
            .object_model_cache
            .set(object_key.to_owned(), fetched_model.clone())
            .await;

        Ok(fetched_model)
    }

    pub async fn get_entity_model(
        &self,
        entity_key: &String,
    ) -> Result<EntityModel, SecurityHandlerResult> {
        if let Some(found) = self
            .cache_container
            .entity_model_cache
            .get(entity_key)
            .await
        {
            return Ok(found);
        }

        let fetched_model = self
            .repo_rest
            .fetch_entitymodel(entity_key.as_str())
            .await
            .map_err(|e| SecurityHandlerResult::RepositoryError(e))?;

        self.cache_container
            .entity_model_cache
            .set(entity_key.to_owned(), fetched_model.clone())
            .await;

        Ok(fetched_model)
    }

    pub async fn get_group_model(
        &self,
        group_user_key: &String,
    ) -> Result<GroupModel, SecurityHandlerResult> {
        if let Some(found) = self
            .cache_container
            .group_model_cache
            .get(group_user_key)
            .await
        {
            return Ok(found);
        }

        let fetched_model = self
            .repo_rest
            .fetch_groupmodel(group_user_key.as_str())
            .await
            .map_err(|e| SecurityHandlerResult::RepositoryError(e))?;

        self.cache_container
            .group_model_cache
            .set(group_user_key.to_owned(), fetched_model.clone())
            .await;

        Ok(fetched_model)
    }

    pub async fn get_user_model(
        &self,
        user_object_key: &String,
    ) -> Result<UserModel, SecurityHandlerResult> {
        if let Some(found) = self
            .cache_container
            .user_model_cache
            .get(user_object_key)
            .await
        {
            return Ok(found);
        }

        let fetched_model = self
            .repo_rest
            .fetch_usermodel(user_object_key.as_str())
            .await
            .map_err(|e| SecurityHandlerResult::RepositoryError(e))?;

        self.cache_container
            .user_model_cache
            .set(user_object_key.to_owned(), fetched_model.clone())
            .await;

        Ok(fetched_model)
    }
}

pub struct RepoRest {
    api_endpoint: String,
}

impl RepoRest {
    pub async fn fetch_usermodel(&self, full_name: &str) -> Result<UserModel, String> {
        // let client = Client::new();
        // let url = format!("{}/{}", self.api_endpoint, guid);
        // let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
        // let model = response.json::<M>().await.map_err(|e| e.to_string())?;
        // Ok(model)
        todo!()
    }

    pub async fn fetch_entitymodel(&self, full_name: &str) -> Result<EntityModel, String> {
        todo!()
    }

    pub async fn fetch_objectmodel(&self, full_name: &str) -> Result<ObjectModel, String> {
        todo!()
    }

    pub async fn fetch_groupmodel(&self, user_guid: &str) -> Result<GroupModel, String> {
        todo!()
    }

    pub async fn generate_data_access_url(&self, data_asset_id: &str) -> Result<String, String> {
        todo!()
    }
}
