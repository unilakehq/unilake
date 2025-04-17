use crate::frontend::tds::codec::token::TokenPreLoginFedAuthRequiredOption;
use crate::frontend::tds::server_context::EncryptionLevel;
use crate::server::FrontendServer;
use std::net::SocketAddr;
use std::sync::Arc;
use unilake_common::error::Result;

pub struct TdsServer {
    ctx: Arc<TdsServerContext>,
}

#[async_trait::async_trait]
impl FrontendServer for TdsServer {
    async fn start(&mut self, bind: SocketAddr) -> Result<SocketAddr> {
        todo!()
    }

    async fn stop(&mut self, graceful: bool) {
        todo!()
    }
}

pub struct TdsServerContext {
    pub server_principal_name: String,
    pub sts_url: String,
    /// The version of the server, as reported by the server. (major, minor, build, sub_build)
    server_version: (u8, u8, u16, u8),
    pub packet_size: u16,
    pub encryption: EncryptionLevel,
    pub encryption_certificate: Option<Vec<u8>>,
    pub fed_auth_options: TokenPreLoginFedAuthRequiredOption,
    pub session_recovery_enabled: bool,
}
