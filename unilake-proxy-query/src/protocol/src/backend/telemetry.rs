pub struct QueryTelemetry {
    proxy_time: i64,
    backend_time: i64,
    records_processed: u64,
    bytes_processed: u64,
    start_time_utc: i64,
    start_backend_time_utc: i64,
    end_time_utc: i64,
    query_id: Option<String>,
}

impl QueryTelemetry {
    /// Initializes a new QueryTelemetry instance with the given query ID.
    /// Starts the timer on initialization
    pub fn new() -> Self {
        QueryTelemetry {
            proxy_time: 0,
            backend_time: 0,
            records_processed: 0,
            bytes_processed: 0,
            start_time_utc: chrono::offset::Utc::now().timestamp(),
            start_backend_time_utc: 0,
            end_time_utc: 0,
            query_id: None,
        }
    }

    pub fn get_proxy_time(&self) -> i64 {
        self.proxy_time
    }

    pub fn get_backend_time(&self) -> i64 {
        self.backend_time
    }

    pub fn get_records_processed(&self) -> u64 {
        self.records_processed
    }

    pub fn get_bytes_processed(&self) -> u64 {
        self.bytes_processed
    }

    pub fn get_start_time_utc(&self) -> i64 {
        self.start_time_utc
    }

    pub fn get_end_time_utc(&self) -> i64 {
        self.end_time_utc
    }

    /// Query id
    pub fn get_query_id(&self) -> Option<&str> {
        self.query_id.as_ref().map(|s| s.as_str())
    }

    pub fn set_query_id(&mut self, query_id: String) {
        assert!(self.query_id.is_none(), "Can only set the query id once");
        self.query_id = Some(query_id);
    }

    pub fn start_backend_timer(&mut self) {
        self.start_backend_time_utc = chrono::offset::Utc::now().timestamp();
    }

    /// Clock the backend time for the query. This should be called after the backend response has been received.  
    /// The backend response time is the time between when the backend received the query and when the response was sent.
    pub fn clock_backend_time(&mut self) {
        self.backend_time = chrono::offset::Utc::now().timestamp() - self.start_backend_time_utc;
    }

    /// Stop the timer for the query. Should be executed after the backend response has been sent
    pub fn end(&mut self) {
        self.end_time_utc = chrono::offset::Utc::now().timestamp();

        // clock proxy time
        self.proxy_time = (self.end_time_utc - self.start_time_utc) - self.backend_time;
    }

    pub fn set_processed_data(&mut self, records: u64, bytes: u64) {
        self.records_processed = records;
        self.bytes_processed = bytes;
    }
}
