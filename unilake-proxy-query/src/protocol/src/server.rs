use std::net::SocketAddr;
use unilake_common::error::Result;

#[async_trait::async_trait]
pub trait Server: Send {
    async fn start(&mut self, bind: SocketAddr) -> Result<SocketAddr>;
}
