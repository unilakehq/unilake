use std::sync::Arc;
use unilake_common::error::Result;
use unilake_common::singleton_instance::GlobalInstance;

struct SessionManager {
    pub max_sessions: usize,
    // pub active_sessions: Arc<RwLock<HashMap<String, Weak<Session>>>>,
}

impl SessionManager {
    pub fn init() -> Result<()> {
        let global_instance = Self::create();
        GlobalInstance::set(global_instance.clone());
        Ok(())
    }

    pub fn create() -> Arc<SessionManager> {
        todo!()
    }

    pub fn instance() -> Arc<SessionManager> {
        GlobalInstance::get()
    }
}
