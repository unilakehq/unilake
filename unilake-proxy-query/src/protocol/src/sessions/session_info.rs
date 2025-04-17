use std::fmt::{Display, Formatter};
use std::time::SystemTime;

pub struct SessionProcessInfo {
    pub state: SessionInfoState,
    pub created_at: SystemTime,
    pub current_query_id: Option<String>,
}

impl SessionProcessInfo {}

impl Default for SessionProcessInfo {
    fn default() -> Self {
        SessionProcessInfo {
            created_at: SystemTime::now(),
            current_query_id: None,
            state: SessionInfoState::Idle,
        }
    }
}

pub enum SessionInfoState {
    Query,
    Aborting,
    Idle,
}

impl Display for SessionInfoState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SessionInfoState::Query => write!(f, "Query"),
            SessionInfoState::Aborting => write!(f, "Aborting"),
            SessionInfoState::Idle => write!(f, "Idle"),
        }
    }
}
