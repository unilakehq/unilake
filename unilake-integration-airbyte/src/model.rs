use std::collections::HashMap;

use arrow2::datatypes::Schema;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct AirbyteCatalog {
    pub streams: Vec<AirbyteStream>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum AirbyteConnectionStatusResult {
    SUCCEEDED,
    FAILED,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirbyteConnectionStatus {
    pub status: AirbyteConnectionStatusResult,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirbyteLogMessage {
    pub level: AirbyteLogLevel,
    pub message: String,
    pub stack_trace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AirbyteLogLevel {
    FATAL,
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum AirbyteMessageType {
    LOG,
    #[default]
    TRACE,
    STATE,
    RECORD,
    SPEC,
    CATALOG,
    #[serde(alias = "CONNECTION_STATUS")]
    CONNECTION,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AirbyteMessage {
    #[serde(alias = "type")]
    pub t: AirbyteMessageType,
    pub record: Option<AirbyteRecordMessage>,
    pub state: Option<AirbyteStateMessage>,
    pub log: Option<AirbyteLogMessage>,
    pub spec: Option<ConnectorSpecification>,
    #[serde(alias = "connectionStatus")]
    pub connection_status: Option<AirbyteConnectionStatus>,
    pub catalog: Option<AirbyteCatalog>,
    pub trace: Option<AirbyteTraceMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirbyteTraceMessage {
    #[serde(alias = "type")]
    pub t: AirbyteTraceMessageType,
    pub emitted_at: f64,
    pub error: Option<AirbyteErrorTraceMessage>,
    pub estimate: Option<AirbyteEstimateTraceMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirbyteEstimateTraceMessage {
    pub name: String,
    #[serde(alias = "type")]
    pub t: String,
    pub namespace: Option<String>,
    pub row_estimate: Option<u64>,
    pub byte_estimate: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirbyteErrorTraceMessage {
    pub message: String,
    pub internal_message: Option<String>,
    pub stack_trace: Option<String>,
    pub failure_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AirbyteTraceMessageType {
    #[serde(alias = "ERROR")]
    Error,
    #[serde(alias = "ESTIMATE")]
    Estimate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirbyteProtocol {
    pub airbyte_message: AirbyteMessage,
    pub configured_airbyte_catalog: ConfiguredAirbyteCatalog,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirbyteRecordMessage {
    pub stream: String,
    pub data: Value,
    pub emitted_at: u64,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirbyteStateMessage {
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirbyteStream {
    pub name: Option<String>,
    pub json_schema: Value,
    pub supported_sync_modes: Option<Vec<SupportedSyncModes>>,
    pub source_defined_cursor: Option<bool>,
    pub default_cursor_field: Option<Vec<String>>,
    pub source_defined_primary_key: Option<Vec<Vec<String>>>,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SupportedSyncModes {
    #[serde(alias = "full_refresh")]
    FullRefresh,
    #[serde(alias = "incremental")]
    Incremental,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSpecification {}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthType {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfiguredAirbyteCatalog {
    pub streams: Vec<ConfiguredAirbyteStream>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfiguredAirbyteStream {
    pub stream: AirbyteStream,
    pub sync_mode: SupportedSyncModes,
    pub cursor_field: Option<Vec<String>>,
    pub destination_sync_mode: DestinationSyncMode,
    pub primary_key: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DestinationSyncMode {
    #[serde(alias = "append")]
    Append,
    #[serde(alias = "overwrite")]
    Overwrite,
    #[serde(alias = "append_dedup")]
    AppendDedup,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectorSpecification {
    pub protocol_version: Option<String>,
    #[serde(alias = "documentationUrl")]
    pub documentation_url: Option<String>,
    #[serde(alias = "changelogUrl")]
    pub changelog_url: Option<String>,
    #[serde(alias = "connectionSpecification")]
    pub connection_specification: HashMap<String, Value>,
    #[serde(alias = "supportsIncremental")]
    pub supports_incremental: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResult {
    pub process_start: u64,
    pub process_end: u64,
    pub process_run_id: String,
    pub process_source_id: String,
    pub data_processed: Option<Vec<ProcessResultDataProcessed>>,
    pub status: ProcessResultStatus,
    pub error_message: Option<String>,
}

impl Default for ProcessResult {
    fn default() -> Self {
        Self {
            process_start: 0,
            process_end: 0,
            process_run_id: "".to_string(),
            process_source_id: "".to_string(),
            data_processed: None,
            status: ProcessResultStatus::SUCCEEDED,
            error_message: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResultDataProcessed {
    pub stream: String,
    pub schema: Option<Schema>,
    pub records: usize,
    pub bytes: usize,
    pub parts_num: usize,
    pub parts_path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProcessResultStatus {
    SUCCEEDED,
    FAILED,
}
