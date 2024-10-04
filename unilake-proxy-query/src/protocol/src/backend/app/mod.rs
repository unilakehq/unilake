use crate::backend::app::generic::ResultSet;
use crate::frontend::error::TdsWireResult;
use crate::frontend::BatchRequest;
use crate::frontend::RpcRequest;
use std::hash::{DefaultHasher, Hasher};

pub mod azdatastudio;
pub mod generic;
// Hashmap with key being the SQL query to search for and a function to execute it
// Make use of: https://docs.rs/ahash/latest/ahash/struct.AHashMap.html
// Generic approaches accommodate the above
// Also: I think this mod should be placed elsewhere (sql perhaps?), or not

pub struct FederatedFrontendHandler {}

impl FederatedFrontendHandler {
    pub fn exec_query(query: &BatchRequest) -> TdsWireResult<Option<ResultSet>> {
        let hash = hash_query(query);
        generic::process(hash, query)
    }

    pub fn exec_rpc(&self, rpc: &RpcRequest) -> TdsWireResult<Option<ResultSet>> {
        todo!()
    }
}

fn hash_query(query: &BatchRequest) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(&query.query.as_bytes());
    hasher.finish()
}

fn hash_rpc(rpc: &RpcRequest) -> String {
    todo!()
}

fn request_ast(request: &str) -> String {
    todo!()
}
