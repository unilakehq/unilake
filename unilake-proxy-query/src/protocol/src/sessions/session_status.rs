use tokio::time::Instant;

pub struct SessionStatus {
    pub session_started_at: Instant,
    pub last_activity_at: Option<Instant>,
}

impl SessionStatus {
    pub(crate) fn activity_finished(&mut self) {
        self.last_activity_at = Some(Instant::now());
    }
}

impl Default for SessionStatus {
    fn default() -> Self {
        SessionStatus {
            session_started_at: Instant::now(),
            last_activity_at: None,
        }
    }
}
