use crate::backend::data::BackendHandler;
use crate::frontend::tds::server_context::ServerContext;
use crate::session::ServerInstanceMessage;
use casbin::DefaultModel;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use ulid::Ulid;
use unilake_common::error::Result;
use unilake_common::singleton_instance::GlobalInstance;
use unilake_security::ABAC_MODEL;

#[async_trait::async_trait]
pub trait Server: Send {
    async fn start(&mut self, bind: SocketAddr) -> Result<SocketAddr>;
    async fn stop(&mut self);
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

    pub fn instance() -> Arc<ServerInstance> {
        GlobalInstance::get()
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
}
