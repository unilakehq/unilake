use crate::backend::app::generic::FedResult;
use crate::frontend::error::TdsWireResult;
use crate::frontend::BatchRequest;
use crate::frontend::RpcRequest;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;

pub mod azdatastudio;
pub mod generic;
// Hashmap with key being the SQL query to search for and a function to execute it
// Make use of: https://docs.rs/ahash/latest/ahash/struct.AHashMap.html
// Generic approaches accommodate the above
// Also: I think this mod should be placed elsewhere (sql perhaps?), or not

pub enum FederatedRequestType<'a> {
    Query(&'a BatchRequest),
    Rpc(&'a RpcRequest),
}

pub struct FederatedFrontendHandler {}

impl FederatedFrontendHandler {
    pub fn exec_request(
        hash: u64,
        request: FederatedRequestType,
    ) -> TdsWireResult<Option<FedResultStream>> {
        Ok(generic::process_static(hash, &request))
    }
}

pub struct FedResultStream {
    it: Pin<Box<dyn Stream<Item = TdsWireResult<FedResult>> + Send>>,
}

impl FedResultStream {
    pub fn new(it: Pin<Box<dyn Stream<Item = TdsWireResult<FedResult>> + Send>>) -> Self {
        Self { it }
    }
}

impl Stream for FedResultStream {
    type Item = TdsWireResult<FedResult>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.it).poll_next(cx)
    }
}
