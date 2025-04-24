use crate::frontend::tds::codec::token::TokenError;
use crate::frontend::tds::codec::{LoginMessage, TdsMessage};
use crate::frontend::tds::prot::{TdsSessionState, TdsWireHandlerFactory};
use crate::frontend::tds::tds_server::TdsServerContext;
use crate::frontend::tds::TdsWireMessageServerCodec;
use crate::sessions::{FeSessionContext, Session, SessionVariable};

use futures::SinkExt;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU16;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::select;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use unilake_common::error::Result;
use unilake_common::error_code::ErrorCode;

pub struct TdsSessionContext {
    // todo: socketaddr can move to session instead
    socket_addr: SocketAddr,
    state: TdsSessionState,
    packet_size: Arc<AtomicU16>,
    sql_user_id: Option<Arc<str>>,
    connection_reset_request_count: usize,
    login_message: Option<LoginMessage>,
    client_nonce: Option<[u8; 32]>,
    server_nonce: Option<[u8; 32]>,
}

impl TdsSessionContext {
    fn state(&self) -> &TdsSessionState {
        &self.state
    }

    fn set_state(&mut self, new_state: TdsSessionState) {
        self.state = new_state;
    }

    fn packet_size(&self) -> Arc<AtomicU16> {
        self.packet_size.clone()
    }

    fn set_sql_user_id(&mut self, sql_user_id: String) {
        self.sql_user_id = Some(Arc::from(sql_user_id));
    }
}

impl FeSessionContext for TdsSessionContext {
    fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    fn get_username(&self) -> Option<Arc<str>> {
        self.sql_user_id.clone()
    }

    fn set_default_variables(&self, variables: &mut HashMap<String, SessionVariable>) {
        todo!()
    }
}

pub struct TdsSession<H: TdsWireHandlerFactory, T: AsyncRead + AsyncWrite + Unpin + Send + Sync> {
    server_ctx: Arc<TdsServerContext>,
    session_ctx: Option<TdsSessionContext>,
    handler: H,
    socket: Framed<T, TdsWireMessageServerCodec>,
    peer_addr: SocketAddr,
}

impl<H, T> TdsSession<H, T>
where
    H: TdsWireHandlerFactory,
    T: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    pub fn new(
        socket: Framed<T, TdsWireMessageServerCodec>,
        peer_addr: SocketAddr,
        server_ctx: Arc<TdsServerContext>,
        handler: H,
    ) -> Self {
        TdsSession {
            session_ctx: None,
            peer_addr,
            socket,
            server_ctx,
            handler,
        }
    }

    pub async fn handle_connection(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        let session = Arc::new(self.handler.open_session(&self.peer_addr).await?);
        let err = select! {
            result = self.process_requests(session.clone()) => {
                result
            }
            _ = cancellation_token.cancelled() => {
                Err(ErrorCode::ServerShutdown("Server termination received, connection closed"))
            }
        };

        if let Err(err) = err {
            self.process_error(err).await?;
        }

        self.handler.close_session(session).await;
        Ok(())
    }

    async fn open_session(&mut self) -> Result<()> {
        // todo: try and open a session here and also register it to the session manager

        todo!()
    }

    async fn process_requests(&mut self, session: Arc<Session>) -> Result<()> {
        while let Some(packet) = self.socket.next().await {
            match packet {
                Ok(msg) => {
                    for (_header, message) in msg.messages {
                        match self.get_session_context()?.state() {
                            TdsSessionState::Initial => {
                                if let TdsMessage::PreLogin(p) = message {
                                    self.handler.on_prelogin_request(&mut self.socket, session.clone(), &p).await?;
                                } else {
                                    return Err(ErrorCode::TdsIncorrectState("PreLogin"));
                                }
                            }
                            TdsSessionState::PreLoginProcessed => {
                                if let TdsMessage::Login(l) = message {
                                    self.handler.on_login7_request(&mut self.socket, session.clone(), &l).await?;
                                    self.get_session_context()?.set_state(TdsSessionState::LoggedIn);
                                } else {
                                    return Err(ErrorCode::TdsIncorrectState("Login"));
                                }
                            }
                            TdsSessionState::SSLNegotiationProcessed => {}
                            TdsSessionState::CompleteLogin7Processed => {}
                            TdsSessionState::Login7SPNEGOProcessed => {}
                            TdsSessionState::Login7FederatedAuthenticationInformationRequestProcessed => {}
                            TdsSessionState::LoggedIn => {
                                if self.get_session_context()?.state() != &TdsSessionState::LoggedIn {
                                    todo!()
                                }

                                if let TdsMessage::BatchRequest(b) = message {
                                    self.handler.on_sql_batch_request(&mut self.socket, session.clone(), &b).await?;
                                } else if let TdsMessage::RemoteProcedureCall(rpc) = message {
                                    self.handler.on_remote_procedure_call(
                                        &mut self.socket,
                                        session.clone(),
                                        &rpc,
                                    )
                                        .await?;
                                } else {
                                    return Err(ErrorCode::TdsIncorrectState("LoggedIn"));
                                }
                            }
                            TdsSessionState::RequestReceived => {}
                            TdsSessionState::AttentionReceived => {}
                            TdsSessionState::ReConnect => {}
                            TdsSessionState::LogoutProcessed => {}
                            TdsSessionState::Final => {}
                        }
                    }

                    self.handler.flush(&mut self.socket).await?;
                    self.socket.flush().await?;
                }
                Err(e) => self.process_error(e).await?,
            }
        }
        Ok(())
    }

    fn get_session_context<'a>(&self) -> Result<&'a mut TdsSessionContext> {
        if let Some(session_ctx) = &self.session_ctx {
            return Ok(&mut session_ctx);
        }
        Err(ErrorCode::Internal("Session context not found".to_string()))
    }

    async fn process_error(&mut self, error: ErrorCode) -> Result<()> {
        self.handler
            .send_token(
                &mut self.socket,
                TokenError::new(
                    error.code() as u32,
                    0,
                    0,
                    error.message(),
                    self.server_ctx.server_name.clone(),
                    "TDS Proxy".to_string(),
                    0,
                ),
            )
            .await
    }
}
