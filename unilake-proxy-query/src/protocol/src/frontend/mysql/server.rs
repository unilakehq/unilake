use crate::server::Server;
use std::net::SocketAddr;
use unilake_common::error::Result;

pub struct MysqlServer;

#[async_trait::async_trait]
impl Server for MysqlServer {
    async fn start(&mut self, bind: SocketAddr) -> Result<SocketAddr> {
        todo!()
    }

    async fn stop(&mut self, graceful: bool) {
        todo!()
    }
}
