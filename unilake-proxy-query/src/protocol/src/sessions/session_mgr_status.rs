use std::time::SystemTime;

#[derive(Clone)]
pub struct SessionManagerStatus {
    pub running_queries_count: u64,
    pub active_sessions_count: u64,
    pub max_running_query_execution_seconds: u64,
    pub last_query_started_at: Option<SystemTime>,
    pub last_query_finished_at: Option<SystemTime>,
    pub instance_started_at: SystemTime,
}

impl SessionManagerStatus {
    pub(crate) fn query_start(&mut self, now: SystemTime) {
        self.running_queries_count += 1;
        self.last_query_started_at = Some(now)
    }

    pub(crate) fn query_finish(&mut self, now: SystemTime) {
        self.running_queries_count = self.running_queries_count.saturating_sub(1);
        if self.running_queries_count == 0 {
            self.last_query_finished_at = Some(now)
        }
    }
}

impl Default for SessionManagerStatus {
    fn default() -> Self {
        SessionManagerStatus {
            running_queries_count: 0,
            active_sessions_count: 0,
            max_running_query_execution_seconds: 0,
            last_query_started_at: None,
            last_query_finished_at: None,
            instance_started_at: SystemTime::now(),
        }
    }
}
