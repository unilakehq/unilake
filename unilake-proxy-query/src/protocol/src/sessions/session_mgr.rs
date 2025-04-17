use crate::sessions::session::Session;
use crate::sessions::session_info::SessionInfoState;
use crate::sessions::session_mgr_status::SessionManagerStatus;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use ulid::Ulid;
use unilake_common::error::Result;
use unilake_common::error_code::ErrorCode;
use unilake_common::singleton_instance::GlobalInstance;

pub struct SessionManager {
    pub max_sessions: usize,
    pub active_sessions: Arc<RwLock<HashMap<Ulid, Weak<Session>>>>,
    pub status: Arc<RwLock<SessionManagerStatus>>,
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

    fn validate_max_session_count(&self, count: usize, reason: &str) -> Result<()> {
        if count > self.max_sessions {
            Err(ErrorCode::TooManySessions(format!(
                "Too many sessions: {} (max: {}) - {}",
                count, self.max_sessions, reason
            )))
        } else {
            Ok(())
        }
    }

    pub fn add_session(&self, session: Arc<Session>) -> Result<()> {
        self.validate_max_session_count(
            self.active_sessions.read().len() + 1,
            &format!("{}", session.typ),
        )?;

        let mut active_sessions = self.active_sessions.write();
        active_sessions.insert(session.get_session_id(), Arc::downgrade(&session));
        Ok(())
    }

    pub fn get_session_by_id(&self, session_id: Ulid) -> Option<Arc<Session>> {
        let active_sessions = self.active_sessions.read();
        active_sessions
            .get(&session_id)
            .and_then(|weak_ptr| weak_ptr.upgrade())
    }

    pub fn destroy_session(&self, session_id: Ulid) -> Result<()> {
        todo!()
    }

    pub fn graceful_shutdown(&self) -> Result<()> {
        todo!()
    }

    pub fn destroy_idle_sessions(&self) -> Result<()> {
        todo!()
    }

    pub fn get_current_session_status(&self) -> SessionManagerStatus {
        let mut status = self.status.read().clone();

        let mut running_queries_count = 0;
        let mut active_sessions_count = 0;
        let mut max_current_query_execution_seconds = 0;

        for session in self.active_sessions_snapshot() {
            if let Some(session_ref) = session.upgrade() {
                active_sessions_count += 1;
                let current_state = session_ref.info.read();
                match current_state.state {
                    SessionInfoState::Query => {
                        running_queries_count += status.running_queries_count;
                        let query_execution_seconds = current_state
                            .created_at
                            .elapsed()
                            .map(|x| x.as_secs())
                            .unwrap_or(0);
                        max_current_query_execution_seconds = std::cmp::max(
                            max_current_query_execution_seconds,
                            query_execution_seconds,
                        );
                    }
                    _ => {}
                }
            }
        }

        status.running_queries_count = running_queries_count;
        status.active_sessions_count = active_sessions_count;
        status.max_running_query_execution_seconds = max_current_query_execution_seconds;
        status
    }

    fn active_sessions_snapshot(&self) -> Vec<Weak<Session>> {
        self.active_sessions.read().values().cloned().collect()
    }
}
