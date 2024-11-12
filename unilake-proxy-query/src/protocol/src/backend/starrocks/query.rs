// todo(mrhamburg): proper query handling, keep track of status and queue status as well, get id from backend and keep track there as well
// we can get the query information from:
//  - https://docs.starrocks.io/docs/administration/management/resource_management/query_queues/#enable-global-query-queues
//  -

use unilake_common::error::TdsWireResult;

// todo: this should have its own connection and check on other connections and queries
struct BackendHandler {}

impl BackendHandler {
    pub fn get_query_status(&self, connection_id: u64) -> TdsWireResult<QueryStatus> {
        todo!()
    }

    pub fn get_connection_status(&self, connection_id: u64) -> TdsWireResult<ProcessInfo> {
        todo!()
    }

    pub fn get_backend_query_id(&self, connection_id: u64) -> TdsWireResult<String> {
        todo!()
    }
}

struct QueryStatus {
    query_id: String,
    resource_group_id: Option<usize>,
    start_time: u64,
    pending_timeout: u64,
    query_timeout: u64,
    state: String,
    slots: usize,
    frontend: String,
    fe_start_time: u64,
}

struct ProcessInfo {
    id: usize,
    user: String,
    host: String,
    db: String,
    command: String,
    connection_start_time: u64,
    time: usize,
    state: String,
    info: String,
    is_pending: bool,
}
