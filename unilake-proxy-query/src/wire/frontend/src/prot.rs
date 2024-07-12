use crate::{codec::TdsWireError, tds::server_context::ServerContext, PreloginMessage};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub enum TdsSessionState {
    #[default]
    /// Sent Initial PRELOGIN Packet State
    PreLoginSent,
    /// Sent TLS/SSL Negotiation Packet State
    SSLNegotiationSent,
    /// Sent LOGIN7 Record with Complete Authentication Token state
    CompleteLogin7Sent,
    /// Sent LOGIN7 Record with SPNEGO Packet State
    Login7SPNEGOSent,
    /// Sent LOGIN7 Record with Authentication information request.
    Login7FederatedAuthenticationInformationRequestSent,
    /// Logged In State
    LoggedIn,
    /// Sent Client Request State
    RequestSent,
    /// Sent Attention State
    AttentionSent,
    /// Indicates that a connection was re-routed to a different SQL Server and transport needs to be re-established
    ReConnect,
    /// Sent a final notification to the TDS Server
    LogoutSent,
    /// Final State
    Final,
}

pub trait SessionInfo {
    /// Currently in use socket
    fn socket_addr(&self) -> SocketAddr;

    /// Current session state
    fn state(&self) -> &TdsSessionState;

    /// Mutate current session state
    fn set_state(&mut self, new_state: TdsSessionState);

    /// Session identifier
    fn session_id(&self) -> usize;

    /// Size of the TDS packet
    fn packet_size(&self) -> usize;

    /// User name if SQL authentication is used
    fn sql_user_id(&self) -> &str;

    /// Database to which connection is established
    fn database(&self) -> &str;

    /// TDS version of the communication
    fn tds_version(&self) -> &str;

    /// TDS server context
    fn tds_server_context(&self) -> Arc<ServerContext>;

    /// Counter of connection reset requests for this session
    fn connection_reset_request_count(&self) -> usize;
}

pub struct DefaultSession {
    socket_addr: SocketAddr,
    state: TdsSessionState,
    session_id: usize,
    packet_size: usize,
    sql_user_id: usize,
    database: Option<String>,
    tds_server_context: Arc<ServerContext>,
}

impl SessionInfo for DefaultSession {
    fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    fn state(&self) -> &TdsSessionState {
        &self.state
    }

    fn set_state(&mut self, new_state: TdsSessionState) {
        self.state = new_state
    }

    fn tds_server_context(&self) -> Arc<ServerContext> {
        self.tds_server_context.clone()
    }

    fn session_id(&self) -> usize {
        todo!()
    }

    fn packet_size(&self) -> usize {
        todo!()
    }

    fn sql_user_id(&self) -> &str {
        todo!()
    }

    fn database(&self) -> &str {
        todo!()
    }

    fn tds_version(&self) -> &str {
        todo!()
    }

    fn connection_reset_request_count(&self) -> usize {
        todo!()
    }
}

pub struct TdsWireMessageServerCodec {
    pub client_info: DefaultSession,
}

impl DefaultSession {
    pub fn new(socket_addr: SocketAddr, ctx: Arc<ServerContext>) -> Self {
        DefaultSession {
            socket_addr,
            packet_size: 1200,
            session_id: 1,
            sql_user_id: 0,
            state: TdsSessionState::default(),
            database: None,
            tds_server_context: ctx,
        }
    }
}

pub trait TdsWireHandlerFactory<S>
where
    S: SessionInfo,
{
    /// Create a new TDS server session
    fn open_session(
        &self,
        socket_addr: &SocketAddr,
        instance_info: &ServerInstance,
    ) -> Result<S, TdsWireError>;

    /// Close TDS server session
    fn close_session(&self, session: &S);

    /// Called when pre-login request arrives
    fn on_prelogin_request(&self, session: &S, msg: &PreloginMessage);

    /// Called when login request arrives
    fn on_login7_request(&self, session: &S);

    /// Called when federated authentication token message arrives. Called only when
    /// such a message arrives in response to federated authentication info, not when the
    /// token is part of a login request.
    fn on_federated_authentication_token_message(&self, session: &S);

    /// Called when SQL batch request arrives
    fn on_sql_batch_request(&self, session: &S);

    /// Called when attention arrives
    fn on_attention(&self, session: &S);
}

pub enum ServerInstanceMessage {
    /// Used to increase the current session counter by one
    IncrementSessionCounter,
    /// Used to decrement the current session counter by one
    DecrementSessionCounter,
    /// Used to send an audit message for a connected session
    Audit(SessionAuditMessage),
}

/// These messages should be forwarded to SIEM/Audit logging endpoint
/// todo(mrhamburg): extend and expand where needed
pub enum SessionAuditMessage {
    /// SqlBatch execution by a user
    SqlBatch(SessionUserInfo, Arc<str>, Arc<str>),
    /// Login succeeded event
    LoginSucceeded(SessionUserInfo),
    /// Login failed event
    LoginFailed(SessionUserInfo),
}

#[allow(unused)]
/// todo(mrhamburg): properly implement and use where applicable
pub struct SessionUserInfo {
    socket_addr: SocketAddr,
    userid: Arc<str>,
}

pub struct ServerInstance {
    active_sessions: usize,
    session_limit: usize,
    #[allow(dead_code)]
    receiver: Option<tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage>>,
    sender: Arc<tokio::sync::mpsc::UnboundedSender<ServerInstanceMessage>>,
    pub ctx: Arc<ServerContext>,
}

impl ServerInstance {
    // todo(mrhamburg): implement logic for processing messages to retraced api or kafka for audit logging and other informational purposes
    pub fn new(ctx: ServerContext) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ServerInstanceMessage>();
        ServerInstance {
            active_sessions: 0,
            session_limit: 32767,
            receiver: Some(receiver),
            sender: Arc::new(sender),
            ctx: Arc::new(ctx),
        }
    }

    pub fn start_instance(&mut self, instance: Arc<RwLock<Self>>) -> tokio::task::JoinHandle<()> {
        async fn run(
            instance: Arc<RwLock<ServerInstance>>,
            mut receiver: tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage>,
        ) {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    ServerInstanceMessage::IncrementSessionCounter => {
                        {
                            instance.write().await.active_sessions += 1;
                        }
                        let current_count = instance.read().await.active_session_count();
                        tracing::info!(
                            message = "Session was added",
                            session_count = current_count
                        );
                    }
                    ServerInstanceMessage::DecrementSessionCounter => {
                        {
                            instance.write().await.active_sessions -= 1;
                        }
                        let current_count = instance.read().await.active_session_count();
                        tracing::info!(
                            message = "Session was dropped",
                            session_count = current_count
                        );
                    }
                    _ => todo!(),
                }
            }
        }
        if self.receiver.is_none() {
            panic!("server instance is already started")
        }
        let r = self.receiver.take().unwrap();
        tokio::spawn(async move { run(instance, r).await })
    }

    pub fn active_session_count(&self) -> usize {
        self.active_sessions
    }

    pub fn session_limit(&self) -> usize {
        self.session_limit
    }

    pub fn get_sender(&self) -> Arc<tokio::sync::mpsc::UnboundedSender<ServerInstanceMessage>> {
        self.sender.clone()
    }
}
