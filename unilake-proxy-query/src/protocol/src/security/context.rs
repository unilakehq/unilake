use std::sync::Arc;

pub struct ConnectionContext {
    pub user_id: Arc<str>,
    pub role: Option<Arc<str>>,
    pub source_application: Arc<str>,
    pub session_id: Arc<str>,
    pub source_ip: Arc<str>,
    pub connection_timestamp: Arc<str>,
    pub endpoint: Arc<str>,
    pub workspace_id: Arc<str>,
    pub compute_id: Arc<str>,
    pub default_catalog: Arc<str>,
    pub default_database: Arc<str>,
    pub branch_name: Arc<str>,
    pub dialect: Arc<str>,
}

pub struct ExecutionContext {
    pub connection_context: ConnectionContext,
    pub query_id: String,
    pub timestamp: String,
    pub input_query: String,
    pub input_query_hash: String,
    pub filtered_query: String,
    pub filtered_query_hash: String,
    pub processing_time_ms: usize,
    pub proxy_time_ms: usize,
    pub record_count: usize,
    pub result_size: usize,
    pub policies_applied: Vec<String>,
}
