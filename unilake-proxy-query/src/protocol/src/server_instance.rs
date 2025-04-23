use crate::backend::backend_data::BackendHandler;
use crate::server_messages::ServerMessageHandler;
use crate::session::ServerInstanceMessage;
use casbin::DefaultModel;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::task;
use tokio::task::{Id, JoinHandle};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use unilake_common::error::Result;
use unilake_common::singleton_instance::GlobalInstance;
use unilake_security::ABAC_MODEL;

// todo: also requires cancellation, tasktracker, and join_handle
pub struct ServerInstance {
    backend_handler: Arc<BackendHandler>,
    default_model: Option<DefaultModel>,
    tracker: TaskTracker,
    cancellation_token: CancellationToken,
    tasks: Arc<RwLock<HashMap<Id, CancellationToken>>>,
    message_handler: Arc<ServerMessageHandler>,
}

impl ServerInstance {
    pub async fn init() -> Result<JoinHandle<()>> {
        let global_instance = Self::create().await;
        GlobalInstance::set(global_instance.clone());

        global_instance.backend_handler.start();
        global_instance.message_handler.start();

        Ok(tokio::spawn(async move {
            global_instance.tracker.wait().await;
        }))
    }

    async fn create() -> Arc<Self> {
        let mut instance = ServerInstance {
            backend_handler: BackendHandler::init(),
            tracker: TaskTracker::new(),
            cancellation_token: CancellationToken::new(),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            default_model: None,
            message_handler: Arc::new(ServerMessageHandler::default()),
        };

        instance.load_abac_model().await;
        Arc::new(instance)
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

    pub fn process_message(
        &self,
        msg: ServerInstanceMessage,
    ) -> std::result::Result<(), tokio::sync::mpsc::error::SendError<ServerInstanceMessage>> {
        self.message_handler.process_message(msg)
    }

    pub fn spawn(&self, join_handle: JoinHandle<()>, ct: CancellationToken) -> Id {
        let tasks = self.tasks.clone();
        let task = self.tracker.spawn(async move {
            {
                tasks.write().insert(task::id(), ct);
            }
            let _ = join_handle.await;
            {
                tasks.write().remove(&task::id());
            }
        });
        task.id()
    }

    pub fn tokio_spawn<F>(&self, future: F, ct: CancellationToken) -> Id
    where
        F: Future<Output = ()> + Send + 'static,
        F::Output: Send + 'static,
    {
        self.spawn(tokio::spawn(future), ct)
    }

    pub fn cancel_task(&self, task_id: &Id) {
        if let Some(ct) = self.tasks.write().get_mut(task_id) {
            ct.cancel();
        }
    }

    pub fn get_cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.child_token()
    }
}
