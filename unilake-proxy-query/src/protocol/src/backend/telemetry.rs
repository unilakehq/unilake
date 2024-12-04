use crate::frontend::prot::{ServerInstance, ServerInstanceMessage};
use crate::frontend::tds::server_context::ServerContext;
use crate::frontend::TokenInfo;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, Clone)]
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
    pub fn new() -> Self {
        QueryTelemetry {
            proxy_time: 0,
            backend_time: 0,
            records_processed: 0,
            bytes_processed: 0,
            start_time_utc: chrono::offset::Utc::now().timestamp_millis(),
            start_backend_time_utc: 0,
            end_time_utc: 0,
            query_id: None,
        }
    }

    pub fn get_proxy_time_in_ms(&self) -> i64 {
        self.proxy_time
    }

    pub fn get_backend_time_in_ms(&self) -> i64 {
        self.backend_time
    }

    pub fn get_total_time_in_ms(&self) -> i64 {
        self.get_proxy_time_in_ms() + self.get_backend_time_in_ms()
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

    pub fn generate_telemetry_message_token(&self, server_context: &ServerContext) -> TokenInfo {
        let message = format!(
            "Backend Time: {} ms, Proxy Time: {} ms, Total Time: {} ms",
            self.get_backend_time_in_ms(),
            self.get_proxy_time_in_ms(),
            self.get_total_time_in_ms()
        );
        TokenInfo::new(server_context, 0, 0, 0, message)
    }
}

pub struct QueryTelemetryHandler {
    query_telemetry: Option<QueryTelemetry>,
    server_instance: Arc<ServerInstance>,
}

impl QueryTelemetryHandler {
    /// Initializes a new QueryTelemetry instance with the given query ID.
    /// Starts the timer on initialization
    pub fn new(server_instance: Arc<ServerInstance>) -> Self {
        QueryTelemetryHandler {
            server_instance,
            query_telemetry: Some(QueryTelemetry::new()),
        }
    }

    fn get_instance(&self) -> &QueryTelemetry {
        assert!(
            self.query_telemetry.is_some(),
            "QueryTelemetry has already been emitted"
        );
        self.query_telemetry.as_ref().unwrap()
    }

    pub fn set_query_id(&mut self, query_id: String) {
        assert!(
            self.get_instance().query_id.is_none(),
            "Can only set the query id once"
        );
        if let Some(instance) = self.query_telemetry.as_mut() {
            instance.query_id = Some(query_id);
        }
    }

    pub fn start_backend_timer(&mut self) {
        if let Some(instance) = self.query_telemetry.as_mut() {
            instance.start_backend_time_utc = chrono::offset::Utc::now().timestamp_millis();
        }
    }

    /// Clock the backend time for the query. This should be called after the backend response has been received.  
    /// The backend response time is the time between when the backend received the query and when the response was sent.
    pub fn clock_backend_time(&mut self) {
        if let Some(instance) = self.query_telemetry.as_mut() {
            instance.backend_time =
                chrono::offset::Utc::now().timestamp_millis() - instance.start_backend_time_utc;
        }
    }

    /// Stop the timer for the query. Should be executed after the backend response has been sent.
    /// Also emits the telemetry message to the server instance.
    /// NOTE: consumes the QueryTelemetry instance, no further processing is allowed.
    pub async fn end(&mut self) -> QueryTelemetry {
        if let Some(mut instance) = self.query_telemetry.take() {
            // set end time
            instance.end_time_utc = chrono::offset::Utc::now().timestamp_millis();

            // clock proxy time
            instance.proxy_time =
                (instance.end_time_utc - instance.start_time_utc) - instance.backend_time;

            // emit telemetry
            if let Err(e) = self
                .server_instance
                .process_message(ServerInstanceMessage::QueryTelemetry(instance.clone()))
            {
                tracing::error!("Failed to send query telemetry: {}", e);
            }

            return instance;
        }

        // panic if the instance has been consumed
        panic!("QueryTelemetry instance has already been consumed")
    }

    /// Set the number of processed records and bytes
    pub fn set_processed_data(&mut self, records: u64, bytes: u64) {
        if let Some(instance) = self.query_telemetry.as_mut() {
            instance.records_processed = records;
            instance.bytes_processed = bytes;
        }
    }
}
