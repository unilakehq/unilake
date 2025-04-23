use crate::frontend::tds::codec::token::{
    TokenDone, TokenError, TokenPreLoginFedAuthRequiredOption,
};
use crate::frontend::tds::prot::TdsWireHandlerFactory;
use crate::frontend::tds::server_context::EncryptionLevel;
use crate::frontend::tds::TdsWireMessageServerCodec;
use crate::server::Server;
use crate::server_instance::ServerInstance;
use crate::sessions::{Session, SessionManager};
use futures::StreamExt;
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use unilake_common::error::Result;
use unilake_common::error_code::ErrorCode;

pub struct TdsServerContext {
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

pub struct TdsServer {
    ctx: Arc<TdsServerContext>,
    cancellation_token: CancellationToken,
    is_running: Mutex<bool>,
    handler: Arc<dyn TdsWireHandlerFactory>,
}

impl TdsServer {
    pub fn new(handler: Handler) -> Self {
        let cancellation_token = ServerInstance::instance().get_cancellation_token();
        let factory = Arc::new(handler);

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
            }),
            is_running: Mutex::new(false),
            cancellation_token,
            handler: factory,
        }
    }

    async fn process_error<T, H>(
        error: ErrorCode,
        socket: &mut Framed<T, TdsWireMessageServerCodec>,
        session: Arc<Session>,
        handlers: Arc<H>,
    ) -> Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + Sync,
        H: TdsWireHandlerFactory,
    {
        handlers
            .send_token(
                socket,
                TokenError::new(
                    error.code() as u32,
                    0,
                    0,
                    error.message(),
                    session.tds_server_context().server_name.clone(),
                    "TDS Proxy".to_string(),
                    0,
                ),
            )
            .await?;

        handlers.send_token(socket, TokenDone::new_error(0)).await?;
        handlers.flush(socket).await?;
        Ok(())
    }

    async fn handle_connection<H>(
        socket: TcpStream,
        ctx: Arc<TdsServerContext>,
        handler: Arc<H>,
        ct: CancellationToken,
    ) where
        H: TdsWireHandlerFactory,
    {
        let mut socket = Framed::new(
            socket,
            TdsWireMessageServerCodec::new(ctx.default_packet_size),
        );

        while let Some(packet) = socket.next().await {
            match packet {
                Ok(p) => {}
                Err(e) => TdsServer::process_error(e, socket, todo!(), handler.clone()).unwrap(),
            }
        }
    }

    // async fn process_socket(tcp_socket: TcpStream) -> Result<()> {
    //     let addr = tcp_socket.peer_addr()?;
    //     tcp_socket.set_nodelay(true)?;
    //
    //     let session_mgr = SessionManager::instance();
    //     let session = handler.open_session(&addr).await?;
    //     let session = session_mgr.add_session(session)?;
    //
    //     let tcp_socket = Framed::new(
    //         tcp_socket,
    //         TdsWireMessageServerCodec::new(session.packet_size()),
    //     );
    //     // let ssl = peek_for_sslrequest(&mut tcp_socket, tls_acceptor.is_some()).await?;
    //
    //     let ssl = false; // todo: implement ssl handshake and check for ssl request
    //     if !ssl {
    //         let mut socket = tcp_socket;
    //
    //         while let Some(packet) = socket.next().await {
    //             match packet {
    //                 Ok(msg) => {
    //                     if let Err(e) =
    //                         process_request(msg, &mut socket, session.clone(), handler.clone())
    //                             .await
    //                     {
    //                         tracing::info!("Error processing request: {}", e);
    //                         process_error(e, &mut socket, session.clone(), handler.clone()).await?;
    //                     }
    //                 }
    //                 Err(e) => {
    //                     tracing::error!("Error reading packet: {}", e);
    //                     // todo(mrhamburg): error handling + close session on error
    //                     // session_info.close_session().await?;
    //                     socket.close().await?;
    //                 }
    //             }
    //         }
    //
    //         // remove session
    //         handler.close_session(session).await;
    //     }

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
        handler: Arc<Handler>,
    ) {
        while let Some(socket) = stream.next().await {
            match socket {
                Ok(socket) => {
                    let ctx = ctx.clone();
                    let ct = ct.child_token();
                    let inner_ct = ct.clone();
                    let handler = handler.clone();
                    ServerInstance::instance().tokio_spawn(
                        async move {
                            TdsServer::<Handler>::handle_connection(socket, ctx, handler, inner_ct)
                                .await;
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
impl<Handler> Server for TdsServer<Handler>
where
    Handler: TdsWireHandlerFactory + 'static,
{
    async fn start(&mut self, bind: SocketAddr) -> Result<SocketAddr> {
        let running = *self.is_running.lock();
        if running {
            return Err(ErrorCode::Internal("TdsServer is already running"));
        }

        let (stream, listener) = self.listener_tcp(bind).await?;
        let ctx = self.ctx.clone();
        let ct = self.cancellation_token.clone();
        let handler = self.handler.clone();

        ServerInstance::instance().tokio_spawn(
            async move {
                TdsServer::listen_loop(stream, ctx, ct, handler).await;
            },
            self.cancellation_token.clone(),
        );

        *self.is_running.get_mut() = true;
        Ok(listener)
    }
}
