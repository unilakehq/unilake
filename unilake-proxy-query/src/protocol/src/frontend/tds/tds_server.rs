use crate::frontend::tds::codec::token::TokenPreLoginFedAuthRequiredOption;
use crate::frontend::tds::codec::TdsFrontendRequest;
use crate::frontend::tds::server_context::EncryptionLevel;
use crate::frontend::tds::{process_error, process_request, TdsWireMessageServerCodec};
use crate::server::{FrontendServer, ListeningStream};
use crate::sessions::SessionManager;
use futures::future::AbortHandle;
use futures::future::{AbortRegistration, Abortable};
use futures::StreamExt;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::Framed;
use unilake_common::error::Result;
use unilake_common::error_code::ErrorCode;

pub struct TdsServer {
    ctx: Arc<TdsServerContext>,
    abort_handle: AbortHandle,
    abort_registration: Option<AbortRegistration>,
    join_handle: Option<JoinHandle<()>>,
}

impl TdsServer {
    pub fn new() -> Self {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
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
            abort_handle,
            abort_registration: Some(abort_registration),
            join_handle: None,
        }
    }

    fn accept_connection(&self, session_mgr: Arc<SessionManager>, socket: TcpStream) {
        executor.spawn(async move {
            let mut socket = Framed::new(
                socket,
                TdsWireMessageServerCodec::new(self.ctx.default_packet_size),
            );

            while let Some(packet) = socket.next().await {
                match packet {
                    Ok(p) => {}
                    Err(e) => {}
                }
            }
        });
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

    async fn listener_tcp(bind: SocketAddr) -> Result<(TcpListenerStream, SocketAddr)> {
        let listener = TcpListener::bind(bind).await.map_err(|e| {
            ErrorCode::TokioError(format!("{}:{} error: {}", bind.ip(), bind.port(), e))
        })?;
        let listener_addr = listener.local_addr()?;
        Ok((TcpListenerStream::new(listener), listener_addr))
    }

    fn listen_loop(&self, stream: ListeningStream) -> impl Future<Output = ()> {
        stream.for_each(move |accept_socket| {
            let sessions = SessionManager::instance();
            async move {
                match accept_socket {
                    Err(err) => tracing::error!("Error accepting socket: {}", err),
                    Ok(socket) => {
                        self.accept_connection(sessions, socket);
                    }
                }
            }
        })
    }
}

#[async_trait::async_trait]
impl FrontendServer for TdsServer {
    async fn start(&mut self, bind: SocketAddr) -> Result<SocketAddr> {
        match self.abort_registration.take() {
            None => Err(ErrorCode::Internal("TdsServer is already running")),
            Some(registration) => {
                let (stream, listener) = TdsServer::listener_tcp(bind).await?;
                let stream = Abortable::new(stream, registration);
                self.join_handle = Some(unilake_common::runtime::spawn(self.listen_loop(stream)));
                Ok(listener)
            }
        }
    }

    async fn stop(&mut self, graceful: bool) {
        if !graceful {
            return;
        }

        self.abort_handle.abort();

        if let Some(join_handle) = self.join_handle.take() {
            if let Err(error) = join_handle.await {
                tracing::error!("Unexpected error during shutdown TDS Server: {}", error);
            }
        }
    }
}

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
