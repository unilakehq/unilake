use crate::backend::data::BackendHandler;
use crate::backend::telemetry::QueryTelemetry;
use crate::frontend::{
    tds::server_context::ServerContext, BatchRequest, LoginMessage, PreloginMessage,
    TdsBackendResponse, TdsMessage, TdsToken,
};
use crate::session::SessionInfo;
use async_trait::async_trait;
use casbin::{Adapter, DefaultModel};
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
use unilake_common::error::{TdsWireError, TdsWireResult};
use unilake_security::handler::SecurityHandler;
use unilake_security::ABAC_MODEL;

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

#[async_trait]
pub trait TdsWireHandlerFactory<S>: Send + Sync
where
    S: SessionInfo + Send + Sync,
{
    /// Create a new TDS server session
    async fn open_session(
        &self,
        socket_addr: &SocketAddr,
        instance_info: Arc<ServerInstance>,
    ) -> Result<S, TdsWireError>;

    /// Close TDS server session
    async fn close_session(&self, session: &mut S);

    /// Called when pre-login request arrives
    async fn on_prelogin_request<C>(
        &self,
        client: &mut C,
        session_info: &mut S,
        msg: &PreloginMessage,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send;

    /// Called when login request arrives
    async fn on_login7_request<C>(
        &self,
        client: &mut C,
        session_info: &mut S,
        msg: &LoginMessage,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send;

    /// Called when federated authentication token message arrives. Called only when
    /// such a message arrives in response to federated authentication info, not when the
    /// token is part of a login request.
    fn on_federated_authentication_token_message(&self, session: &S);

    /// Called when SQL batch request arrives
    async fn on_sql_batch_request<C>(
        &self,
        client: &mut C,
        session_info: &mut S,
        msg: &BatchRequest,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send;

    /// Called when attention arrives
    fn on_attention(&self, session: &S);

    /// Send message to the client
    async fn send_message<C, M>(&self, client: &mut C, msg: M) -> Result<(), TdsWireError>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
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
        C: Sink<TdsBackendResponse> + Unpin + Send,
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
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        client
            .send(TdsBackendResponse::Done)
            .await
            .map_err(|_| TdsWireError::Protocol("Failed to feed completion".to_string()))
    }
}

pub enum ServerInstanceMessage {
    /// Used to send an audit message for a connected session
    Audit(SessionAuditMessage),
    /// Used to send telemetry data
    Telemetry,
    /// Used to send activity data (Connection)
    ActivityConnection(String),
    /// Used to send Query Telemetry data
    QueryTelemetry(QueryTelemetry),
}

/// These messages should be forwarded to SIEM/Audit logging endpoint
/// todo(mrhamburg): extend and expand where needed
pub enum SessionAuditMessage {
    /// Sql execution by a user
    SqlQuery(SessionUserInfo, SecurityHandler),
    /// Login succeeded event
    LoginSucceeded(SessionUserInfo),
    /// Login failed event
    LoginFailed(SessionUserInfo),
}

pub struct SessionUserInfo {
    socket_addr: SocketAddr,
    userid: String,
}

impl SessionUserInfo {
    pub fn from(info: &dyn SessionInfo) -> Self {
        SessionUserInfo {
            socket_addr: info.socket_addr(),
            userid: info.get_sql_user_id().to_string(),
        }
    }
}

impl From<&dyn SessionInfo> for SessionUserInfo {
    fn from(info: &dyn SessionInfo) -> Self {
        SessionUserInfo {
            socket_addr: info.socket_addr(),
            userid: info.get_sql_user_id().to_string(),
        }
    }
}

pub struct ServerInstance {
    pub ctx: Arc<ServerContext>,
    pub backend_handler: Arc<BackendHandler>,
    default_model: Option<DefaultModel>,
    inner: InnerServerInstance,
}

pub struct InnerServerInstance {
    receiver: Option<tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage>>,
    sender: Arc<tokio::sync::mpsc::UnboundedSender<ServerInstanceMessage>>,
    active_sessions: AtomicUsize,
    semaphore: Arc<Semaphore>,
}

// todo: we might as well put serverinstance in own folder and handle all of this there (coordination of actions for example)
impl ServerInstance {
    // todo(mrhamburg): implement logic for processing messages to retraced api or kafka for audit logging and other informational purposes
    pub fn new(ctx: ServerContext) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ServerInstanceMessage>();
        ServerInstance {
            ctx: Arc::new(ctx),
            backend_handler: Arc::new(BackendHandler::new()),
            inner: InnerServerInstance {
                receiver: Some(receiver),
                sender: Arc::new(sender),
                active_sessions: AtomicUsize::new(0),
                semaphore: Arc::new(Semaphore::new(4)),
            },
            default_model: None,
        }
    }

    /// Load ABAC model from a string and initialize the server instance with it.
    pub async fn load_abac_model(&mut self) {
        self.default_model = Some(DefaultModel::from_str(ABAC_MODEL).await.unwrap())
    }

    pub fn get_abac_model(&self) -> Option<DefaultModel> {
        // expect cloning to be faster than re-initializing the model, since casbin takes ownership of the model we can't reference it
        self.default_model.clone()
    }

    async fn inner_process_message(&self, _msg: ServerInstanceMessage) {
        tracing::error!(message = "Received server instance message, processing has not been implemented, dropping message!".to_string());
    }

    /// Starts the background job server instance for processing server messages.
    /// Currently, is set to max 4 messages being processed in parallel.
    /// In case 4 messages are already being processed, the process will check every 10 milliseconds for
    /// an open slot to process new messages.
    /// Note: the server instance can only be started once, will panic in case the background process has
    /// already been started
    pub async fn start_instance(mut self) -> (Arc<Self>, tokio::task::JoinHandle<()>) {
        tracing::info!(
            "Starting server instance background jobs (SSE consumer, Background Workers({}))",
            self.inner.semaphore.available_permits()
        );

        // also start the sse cache handler
        BackendHandler::start_sse_consumer(self.backend_handler.clone()).await;

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

        tracing::info!("Server instance background jobs started");
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

    pub fn get_policy_adapter(&self) -> Arc<Box<dyn Adapter>> {
        todo!()
    }
}
