use crate::caching::layered_cache::MultiLayeredCache;
use async_trait::async_trait;
use casbin::{Adapter, Filter, Model, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use unilake_common::model::PolicyRule;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CacheEntity {
    Policy(Vec<PolicyRule>),
    PolicyId(u64),
}

pub struct CachedAdapter {
    is_filtered: bool,
    cache: Arc<MultiLayeredCache<u64, CacheEntity>>,
}

impl CachedAdapter {
    pub fn new(cache: Arc<MultiLayeredCache<u64, CacheEntity>>) -> Self {
        CachedAdapter {
            is_filtered: false,
            cache,
        }
    }

    pub async fn get_current_policy_id(&self) -> Option<u64> {
        self.cache.get(&0).await.map(|entity| match entity {
            CacheEntity::PolicyId(id) => id,
            _ => unreachable!(),
        })
    }
}
#[async_trait]
impl Adapter for CachedAdapter {
    async fn load_policy(&mut self, m: &mut dyn Model) -> Result<()> {
        let current_policy_id = self.get_current_policy_id().await.map_or(0, |id| id);
        let current_policy = self
            .cache
            .get(&current_policy_id)
            .await
            .map(|entity| match entity {
                CacheEntity::Policy(policy) => policy.clone(),
                _ => Vec::new(),
            });

        if current_policy.is_none() {
            // return Err(casbin::Error::new("No current policy found")?);
            todo!("Proper error handling logic here, cannot find current policy")
        }

        let current_policy = current_policy.unwrap();
        for line in current_policy {
            let parts = line.to_vec();

            let key = &parts[0];
            if let Some(ref sec) = key.chars().next().map(|x| x.to_string()) {
                if let Some(ast_map) = m.get_mut_model().get_mut(sec) {
                    if let Some(ast) = ast_map.get_mut(key) {
                        ast.policy.insert(parts[1..].to_vec());
                    }
                }
            }
        }

        Ok(())
    }

    async fn load_filtered_policy<'a>(&mut self, _m: &mut dyn Model, _f: Filter<'a>) -> Result<()> {
        unreachable!("this api shouldn't implement, just for convenience")
    }

    async fn save_policy(&mut self, _m: &mut dyn Model) -> Result<()> {
        // this api shouldn't implement, just for convenience
        Ok(())
    }

    async fn clear_policy(&mut self) -> Result<()> {
        // this api shouldn't implement, just for convenience
        Ok(())
    }

    fn is_filtered(&self) -> bool {
        self.is_filtered
    }

    async fn add_policy(&mut self, _sec: &str, _ptype: &str, _rule: Vec<String>) -> Result<bool> {
        // this api shouldn't implement, just for convenience
        Ok(true)
    }

    async fn add_policies(
        &mut self,
        _sec: &str,
        _ptype: &str,
        _rules: Vec<Vec<String>>,
    ) -> Result<bool> {
        // this api shouldn't implement, just for convenience
        Ok(true)
    }

    async fn remove_policy(
        &mut self,
        _sec: &str,
        _ptype: &str,
        _rule: Vec<String>,
    ) -> Result<bool> {
        // this api shouldn't implement, just for convenience
        Ok(true)
    }

    async fn remove_policies(
        &mut self,
        _sec: &str,
        _ptype: &str,
        _rule: Vec<Vec<String>>,
    ) -> Result<bool> {
        // this api shouldn't implement, just for convenience
        Ok(true)
    }

    async fn remove_filtered_policy(
        &mut self,
        _sec: &str,
        _ptype: &str,
        _field_index: usize,
        _field_values: Vec<String>,
    ) -> Result<bool> {
        // this api shouldn't implement, just for convenience
        Ok(true)
    }
}
