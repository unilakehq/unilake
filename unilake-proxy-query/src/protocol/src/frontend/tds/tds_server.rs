use crate::backend::starrocks::frontend_tds::StarRocksTdsHandlerFactory;
use crate::frontend::tds::codec::token::TokenPreLoginFedAuthRequiredOption;
use crate::frontend::tds::prot::TdsWireHandlerFactory;
use crate::frontend::tds::server_context::EncryptionLevel;
use crate::frontend::tds::tds_session::TdsSession;
use crate::frontend::tds::TdsWireMessageServerCodec;
use crate::server::Server;
use crate::server_instance::ServerInstance;

use futures::StreamExt;
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use unilake_common::error::Result;
use unilake_common::error_code::ErrorCode;

pub struct TdsServerContext {
    pub server_name: String,
    pub server_principal_name: String,
    pub sts_url: String,
    /// The version of the server, as reported by the server. (major, minor, build, sub_build)
    server_version: (u8, u8, u16, u8),
    pub default_packet_size: u16,
    pub encryption: EncryptionLevel,
    pub encryption_certificate: Option<Vec<u8>>,
    pub fed_auth_options: TokenPreLoginFedAuthRequiredOption,
    pub session_recovery_enabled: bool,
}

pub struct TdsServer<H: TdsWireHandlerFactory> {
    ctx: Arc<TdsServerContext>,
    cancellation_token: CancellationToken,
    is_running: Mutex<bool>,
    handler: Arc<H>,
}

impl<H> TdsServer<H>
where
    H: TdsWireHandlerFactory + 'static,
{
    pub fn new(handler: H) -> Self {
        let cancellation_token = ServerInstance::instance().get_cancellation_token();
        let handler = Arc::new(handler);
        let server_instance = ServerInstance::instance();

        TdsServer {
            ctx: Arc::new(TdsServerContext {
                server_principal_name: "unilake".to_string(),
                sts_url: "sts://unilake.com/token".to_string(),
                server_version: (1, 0, 0, 0),
                default_packet_size: 4096,
                encryption: EncryptionLevel::Off,
                encryption_certificate: None,
                fed_auth_options: TokenPreLoginFedAuthRequiredOption::FedAuthNotRequired,
                session_recovery_enabled: false,
                server_name: server_instance.get_server_name().clone(),
            }),
            is_running: Mutex::new(false),
            handler,
            cancellation_token,
        }
    }

    async fn handle_connection(
        socket: TcpStream,
        ctx: Arc<TdsServerContext>,
        handler: H,
        ct: CancellationToken,
    ) {
        let peer_addr = socket.peer_addr().unwrap();
        let socket = Framed::new(
            socket,
            TdsWireMessageServerCodec::new(ctx.default_packet_size),
        );

        let mut session = TdsSession::new(socket, peer_addr, ctx.clone(), handler);
        session.handle_connection(ct).await;
    }

    async fn listener_tcp(&self, bind: SocketAddr) -> Result<(TcpListenerStream, SocketAddr)> {
        let listener = TcpListener::bind(bind).await.map_err(|e| {
            ErrorCode::TokioError(format!("{}:{} error: {}", bind.ip(), bind.port(), e))
        })?;
        let listener_addr = listener.local_addr()?;
        Ok((TcpListenerStream::new(listener), listener_addr))
    }

    async fn listen_loop(
        mut stream: TcpListenerStream,
        ctx: Arc<TdsServerContext>,
        ct: CancellationToken,
    ) {
        while let Some(socket) = stream.next().await {
            match socket {
                Ok(socket) => {
                    let ctx = ctx.clone();
                    let ct = ct.child_token();
                    let inner_ct = ct.clone();
                    let handler = H::new();

                    ServerInstance::instance().tokio_spawn(
                        async move {
                            TdsServer::handle_connection(socket, ctx, handler, inner_ct).await;
                        },
                        ct,
                    );
                }
                Err(e) => {
                    tracing::error!("Error accepting connection: {}", e);
                }
            }
        }
        tracing::info!("Server stopped");
    }
}

#[async_trait::async_trait]
impl Server for TdsServer<StarRocksTdsHandlerFactory> {
    async fn start(&mut self, bind: SocketAddr) -> Result<SocketAddr> {
        let running = *self.is_running.lock();
        if running {
            return Err(ErrorCode::Internal("TdsServer is already running"));
        }

        let (stream, listener) = self.listener_tcp(bind).await?;
        let ctx = self.ctx.clone();
        let ct = self.cancellation_token.clone();

        ServerInstance::instance().tokio_spawn(
            async move {
                TdsServer::<StarRocksTdsHandlerFactory>::listen_loop(stream, ctx, ct).await;
            },
            self.cancellation_token.clone(),
        );

        *self.is_running.get_mut() = true;
        Ok(listener)
    }
}
