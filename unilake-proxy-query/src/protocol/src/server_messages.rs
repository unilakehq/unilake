use crate::server_instance::ServerInstance;
// todo: move here?
use crate::session::ServerInstanceMessage;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

pub struct ServerMessageHandler {
    receiver: Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage>>>,
    sender: Arc<tokio::sync::mpsc::UnboundedSender<ServerInstanceMessage>>,
    semaphore: Arc<Semaphore>,
    inner: Arc<InnerServerMessageHandler>,
}

impl ServerMessageHandler {
    pub fn process_message(
        &self,
        msg: ServerInstanceMessage,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<ServerInstanceMessage>> {
        self.sender.clone().send(msg)
    }

    fn get_receiver(&self) -> tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage> {
        let mut receiver = self.receiver.lock();
        if receiver.is_none() {
            panic!("Server instance is already started")
        }
        receiver.take().unwrap()
    }

    pub fn start(&self) {
        tracing::info!(
            "Starting server instance background jobs (SSE consumer, Background Workers({}))",
            self.semaphore.available_permits()
        );

        let semaphore = self.semaphore.clone();
        let handler = self.inner.clone();
        let mut receiver = self.get_receiver();

        let join_handle = tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                let handler = handler.clone();
                let semaphore = semaphore.clone();

                tokio::spawn(async move {
                    let semaphore = semaphore.acquire().await.unwrap();
                    handler.process_message(msg).await;
                    drop(semaphore);
                });
            }
        });

        ServerInstance::instance().spawn(join_handle, CancellationToken::new());
    }
}

impl Default for ServerMessageHandler {
    fn default() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ServerInstanceMessage>();
        ServerMessageHandler {
            receiver: Mutex::new(Some(receiver)),
            sender: Arc::new(sender),
            // todo: get 4 from env var/config
            semaphore: Arc::new(Semaphore::new(4)),
            inner: Arc::new(InnerServerMessageHandler),
        }
    }
}

struct InnerServerMessageHandler;
impl InnerServerMessageHandler {
    pub async fn process_message(&self, _msg: ServerInstanceMessage) {
        tracing::error!(message = "Received server instance message, processing has not been implemented, dropping message!".to_string());
    }
}
