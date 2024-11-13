// TODO: implementation for acquiring policy information (PIP)
// Note: use backon for resilience
use unilake_common::model::{EntityModel, GroupModel, ObjectModel, UserModel};

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
