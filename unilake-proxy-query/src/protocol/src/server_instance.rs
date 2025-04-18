use crate::backend::data::BackendHandler;
use crate::session::ServerInstanceMessage;
use casbin::DefaultModel;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use unilake_common::error::Result;
use unilake_common::singleton_instance::GlobalInstance;
use unilake_security::ABAC_MODEL;

// todo: also requires abort_handle, abort_registration, and join_handle
pub struct ServerInstance {
    backend_handler: Arc<BackendHandler>,
    default_model: Option<DefaultModel>,
    inner: InnerServerInstance,
}

pub struct InnerServerInstance {
    receiver: Option<tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage>>,
    sender: Arc<tokio::sync::mpsc::UnboundedSender<ServerInstanceMessage>>,
    semaphore: Arc<Semaphore>,
}

impl ServerInstance {
    pub async fn init() -> Result<tokio::task::JoinHandle<()>> {
        let (global_instance, join_handle) = Self::create().await;
        GlobalInstance::set(global_instance.clone());
        Ok(join_handle)
    }

    async fn create() -> (Arc<Self>, tokio::task::JoinHandle<()>) {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ServerInstanceMessage>();
        let mut instance = ServerInstance {
            backend_handler: Arc::new(BackendHandler::new()),
            inner: InnerServerInstance {
                receiver: Some(receiver),
                sender: Arc::new(sender),
                semaphore: Arc::new(Semaphore::new(4)),
            },
            default_model: None,
        };

        instance.load_abac_model().await;
        instance.start().await
    }

    pub fn instance() -> Arc<ServerInstance> {
        GlobalInstance::get()
    }

    /// Load ABAC model from a string and initialize the server instance with it.
    async fn load_abac_model(&mut self) {
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
    async fn start(mut self) -> (Arc<Self>, tokio::task::JoinHandle<()>) {
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

    pub fn process_message(
        &self,
        msg: ServerInstanceMessage,
    ) -> std::result::Result<(), tokio::sync::mpsc::error::SendError<ServerInstanceMessage>> {
        self.inner.sender.clone().send(msg)
    }
}
