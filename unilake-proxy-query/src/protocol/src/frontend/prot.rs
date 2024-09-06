use crate::frontend::{
    error::{TdsWireError, TdsWireResult},
    tds::server_context::ServerContext,
    BatchRequest, LoginMessage, PreloginMessage, TdsBackendResponse, TdsMessage, TdsToken,
};
use async_trait::async_trait;
use futures::{Sink, SinkExt};
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{sync::Semaphore, time::sleep};
use ulid::Ulid;

#[derive(Debug, Default)]
pub enum TdsSessionState {
    #[default]
    /// Initial State
    Initial,
    /// Received Initial PRELOGIN Packet State
    PreLoginProcessed,
    /// Received TLS/SSL Negotiation Packet State
    SSLNegotiationProcessed,
    /// Received LOGIN7 Record with Complete Authentication Token state
    CompleteLogin7Processed,
    /// Received LOGIN7 Record with SPNEGO Packet State
    Login7SPNEGOProcessed,
    /// Received LOGIN7 Record with Authentication information request.
    Login7FederatedAuthenticationInformationRequestProcessed,
    /// Logged In State
    LoggedIn,
    /// Received Client Request State
    RequestReceived,
    /// Received Attention State
    AttentionReceived,
    /// Indicates that a connection was re-routed to a different SQL Server and transport needs to be re-established
    ReConnect,
    /// Received a final notification to the TDS Server
    LogoutProcessed,
    /// Final State
    Final,
}

pub trait SessionInfo: Send + Sync {
    /// Currently in use socket
    fn socket_addr(&self) -> SocketAddr;

    /// Current session state
    fn state(&self) -> &TdsSessionState;

    /// Mutate current session state
    fn set_state(&mut self, new_state: TdsSessionState);

    /// Session identifier
    fn session_id(&self) -> Ulid;

    /// Size of the TDS packet
    fn packet_size(&self) -> u16;

    /// User name if SQL authentication is used
    fn get_sql_user_id(&self) -> &str;

    /// User name if SQL authentication is used
    fn set_sql_user_id(&mut self, sql_user_id: String);

    /// Database to which connection is established
    fn get_database(&self) -> &str;

    /// Set database to which connection is established
    fn set_database(&mut self, db_name: String);

    /// TDS version of the communication
    fn tds_version(&self) -> &str;

    /// TDS server context
    fn tds_server_context(&self) -> Arc<ServerContext>;

    /// Counter of connection reset requests for this session
    fn connection_reset_request_count(&self) -> usize;

    /// Set client nonce for SQL authentication
    fn set_client_nonce(&mut self, nonce: [u8; 32]);

    /// Get client nonce for SQL authentication
    fn get_client_nonce(&self) -> Option<[u8; 32]>;

    /// Set server nonce for SQL authentication
    fn set_server_nonce(&mut self, nonce: [u8; 32]);

    /// Get server nonce for SQL authentication
    fn get_server_nonce(&self) -> Option<[u8; 32]>;
}

pub enum Dialect {
    Mssql,
}

pub struct DefaultSession {
    socket_addr: SocketAddr,
    state: TdsSessionState,
    session_id: Ulid,
    packet_size: u16,
    sql_user_id: Option<String>,
    database: Option<String>,
    // connection_reset_request_count: usize,
    // dialect: Dialect,
    tds_server_context: Arc<ServerContext>,
    client_nonce: Option<[u8; 32]>,
    server_nonce: Option<[u8; 32]>,
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

    fn session_id(&self) -> Ulid {
        self.session_id
    }

    fn packet_size(&self) -> u16 {
        self.packet_size
    }

    fn get_sql_user_id(&self) -> &str {
        todo!()
    }

    fn get_database(&self) -> &str {
        self.database.as_deref().unwrap_or("")
    }

    fn tds_version(&self) -> &str {
        todo!()
    }

    fn connection_reset_request_count(&self) -> usize {
        todo!()
    }

    fn set_client_nonce(&mut self, nonce: [u8; 32]) {
        self.client_nonce = Some(nonce);
    }

    fn get_client_nonce(&self) -> Option<[u8; 32]> {
        self.client_nonce
    }

    fn set_server_nonce(&mut self, nonce: [u8; 32]) {
        self.server_nonce = Some(nonce);
    }

    fn get_server_nonce(&self) -> Option<[u8; 32]> {
        self.server_nonce
    }

    fn set_sql_user_id(&mut self, sql_user_id: String) {
        self.sql_user_id = Some(sql_user_id);
    }

    fn set_database(&mut self, db_name: String) {
        self.database = Some(db_name);
    }
}

pub struct TdsWireMessageServerCodec {
    pub client_info: DefaultSession,
}

impl DefaultSession {
    pub fn new(socket_addr: SocketAddr, instance: Arc<ServerInstance>) -> Self {
        DefaultSession {
            socket_addr,
            packet_size: instance.ctx.packet_size,
            session_id: instance.next_session_id(),
            sql_user_id: None,
            state: TdsSessionState::default(),
            database: None,
            tds_server_context: instance.ctx.clone(),
            client_nonce: None,
            server_nonce: None,
            // connection_reset_request_count: 0,
            // dialect: Dialect::Mssql,
        }
    }
}

#[async_trait]
pub trait TdsWireHandlerFactory<S>: Send + Sync
where
    S: SessionInfo + Send + Sync,
{
    /// Create a new TDS server session
    fn open_session(
        &self,
        socket_addr: &SocketAddr,
        instance_info: Arc<ServerInstance>,
    ) -> Result<S, TdsWireError>;

    /// Close TDS server session
    fn close_session(&self, session: &S);

    /// Called when pre-login request arrives
    async fn on_prelogin_request<C>(
        &self,
        client: &mut C,
        msg: &PreloginMessage,
    ) -> TdsWireResult<()>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send;

    /// Called when login request arrives
    async fn on_login7_request<C>(&self, client: &mut C, msg: &LoginMessage) -> TdsWireResult<()>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send;

    /// Called when federated authentication token message arrives. Called only when
    /// such a message arrives in response to federated authentication info, not when the
    /// token is part of a login request.
    fn on_federated_authentication_token_message(&self, session: &S);

    /// Called when SQL batch request arrives
    async fn on_sql_batch_request<C>(
        &self,
        client: &mut C,
        msg: &BatchRequest,
    ) -> TdsWireResult<()>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send;

    /// Called when attention arrives
    fn on_attention(&self, session: &S);

    /// Send message to the client
    async fn send_message<C, M>(&self, client: &mut C, msg: M) -> Result<(), TdsWireError>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
        M: Into<TdsMessage> + Send,
    {
        client
            .send(TdsBackendResponse::Message(msg.into()))
            .await
            .map_err(|_| TdsWireError::Protocol("Failed to feed message".to_string()))
    }

    /// Send token to the client
    async fn send_token<C, T>(&self, client: &mut C, token: T) -> Result<(), TdsWireError>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
        T: Into<TdsToken> + Send,
    {
        client
            .send(TdsBackendResponse::Token(token.into()))
            .await
            .map_err(|_| TdsWireError::Protocol("Failed to feed token".to_string()))
    }

    /// Flush all results
    async fn flush<C>(&self, client: &mut C) -> Result<(), TdsWireError>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
    {
        client
            .send(TdsBackendResponse::Done)
            .await
            .map_err(|_| TdsWireError::Protocol("Failed to feed completion".to_string()))
    }
}

#[derive(Debug)]
pub enum ServerInstanceMessage {
    /// Used to send an audit message for a connected session
    Audit(SessionAuditMessage),
    /// Used to send telemetry data
    Telemetry,
    /// Used to send activity data (Connection)
    ActivityConnection(String),
}

/// These messages should be forwarded to SIEM/Audit logging endpoint
/// todo(mrhamburg): extend and expand where needed
#[derive(Debug)]
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
#[derive(Debug)]
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
    active_sessions: AtomicUsize,
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
                active_sessions: AtomicUsize::new(0),
                semaphore: Arc::new(Semaphore::new(4)),
            },
        }
    }

    async fn inner_process_message(&self, msg: ServerInstanceMessage) {
        tracing::error!(message = format!("Received server instance message, processing has not been implemented, dropping message! {:?}", msg));
    }

    /// Starts the background job server instance for processing server messages.
    /// Currently is set to max 4 messages being processed in parallel.
    /// In case 4 messages are already being processed, the process will check every 10 milliseconds for
    /// an open slot to process new messages.
    /// Note: the server instance can only be started once, will panic in case the background process has
    /// already been started
    pub fn start_instance(mut self) -> (Arc<Self>, tokio::task::JoinHandle<()>) {
        async fn run(
            instance: Arc<ServerInstance>,
            mut receiver: tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage>,
        ) {
            while let Some(msg) = receiver.recv().await {
                let instance = instance.clone();
                let semaphore = instance.inner.semaphore.clone();
                while semaphore.available_permits() == 0 {
                    sleep(Duration::from_millis(10)).await;
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
        let instance = Arc::new(self);
        let running_instance = instance.clone();
        (
            instance,
            tokio::spawn(async move { run(running_instance, r).await }),
        )
    }

    pub fn active_session_count(&self) -> usize {
        self.inner.active_sessions.load(Ordering::Relaxed)
    }

    pub fn increment_session_counter(&self) -> usize {
        self.inner.active_sessions.fetch_add(1, Ordering::Relaxed);
        let count = self.active_session_count();
        tracing::info!(
            message = "Increased session count",
            current_count = count,
            max_count = self.session_limit()
        );
        count
    }

    pub fn decrement_session_counter(&self) -> usize {
        self.inner.active_sessions.fetch_sub(1, Ordering::Relaxed);
        let count = self.active_session_count();
        tracing::info!(
            message = "Decreased session count",
            current_count = count,
            max_count = self.session_limit()
        );
        count
    }

    pub fn session_limit(&self) -> usize {
        self.ctx.session_limit
    }

    pub fn next_session_id(&self) -> Ulid {
        let session_id = Ulid::new();
        tracing::trace!("Generating new session ID: {}", session_id.to_string());
        session_id
    }

    pub fn process_message(
        &self,
        msg: ServerInstanceMessage,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<ServerInstanceMessage>> {
        self.inner.sender.clone().send(msg)
    }
}
