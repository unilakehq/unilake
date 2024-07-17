use crate::{codec::TdsWireError, tds::server_context::ServerContext, PreloginMessage};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    sync::{RwLock, Semaphore},
    time::sleep,
};

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
    /// Used to send an audit message for a connected session
    Audit(SessionAuditMessage),
    /// Used to send telemetry data
    Telemetry,
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
    pub ctx: Arc<ServerContext>,
    inner: InnerServerInstance,
}

pub struct InnerServerInstance {
    receiver: Option<tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage>>,
    sender: Arc<tokio::sync::mpsc::UnboundedSender<ServerInstanceMessage>>,
    active_sessions: RwLock<usize>,
    semaphore: Arc<Semaphore>,
}

impl ServerInstance {
    // todo(mrhamburg): implement logic for processing messages to retraced api or kafka for audit logging and other informational purposes
    pub fn new(ctx: ServerContext) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ServerInstanceMessage>();
        ServerInstance {
            ctx: Arc::new(ctx),
            inner: InnerServerInstance {
                receiver: Some(receiver),
                sender: Arc::new(sender),
                active_sessions: RwLock::new(0),
                semaphore: Arc::new(Semaphore::new(4)),
            },
        }
    }

    async fn inner_process_message(&self, msg: ServerInstanceMessage) {
        tracing::error!(message = "Received server instance message, processing has not been implemented, dropping message!");
    }

    /// Starts the background job server instance for processing server messages.
    /// Currently is set to max 4 messages being processed in parallel.
    /// In case 4 messages are already being processed, the process will check every 10 milliseconds for
    /// an open slot to process new messages.
    /// Note: the server instance can only be started once, will panic in case the background process has
    /// already been started
    pub fn start_instance(&mut self, instance: Arc<Self>) -> tokio::task::JoinHandle<()> {
        async fn run(
            instance: Arc<ServerInstance>,
            mut receiver: tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage>,
        ) {
            while let Some(msg) = receiver.recv().await {
                let instance = instance.clone();
                let semaphore = instance.inner.semaphore.clone();
                while semaphore.available_permits() == 0 {
                    sleep(Duration::from_millis(10));
                }
                tokio::task::spawn(async move {
                    let semaphore = semaphore.acquire().await.unwrap();
                    instance.inner_process_message(msg).await;
                    drop(semaphore);
                });
            }
        }

        if self.inner.receiver.is_none() {
            panic!("server instance is already started")
        }
        let r = self.inner.receiver.take().unwrap();
        tokio::spawn(async move { run(instance, r).await })
    }

    pub async fn active_session_count(&self) -> usize {
        self.inner.active_sessions.read().await.clone()
    }

    pub async fn increment_session_counter(&self) -> usize {
        let guard = self.inner.active_sessions.write().await;
        guard.saturating_add(1)
    }

    pub async fn decrement_session_counter(&self) -> usize {
        let guard = self.inner.active_sessions.write().await;
        guard.saturating_sub(1)
    }

    pub fn session_limit(&self) -> usize {
        self.ctx.session_limit
    }

    pub fn process_message(
        &self,
        msg: ServerInstanceMessage,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<ServerInstanceMessage>> {
        self.inner.sender.clone().send(msg)
    }
}
