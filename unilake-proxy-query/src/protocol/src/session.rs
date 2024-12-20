use crate::frontend::prot::TdsSessionState;
use crate::frontend::tds::server_context::ServerContext;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU16;
use std::sync::Arc;
use ulid::Ulid;

pub const SESSION_VARIABLE_DIALECT: &str = "proxy_dialect";
pub const SESSION_VARIABLE_CATALOG: &str = "proxy_catalog";
pub const SESSION_VARIABLE_DATABASE: &str = "proxy_database";
pub const SESSION_VARIABLE_SECURITY_IMPERSONATE: &str = "proxy_security_impersonate";
pub const SESSION_VARIABLE_SEND_TELEMETRY: &str = "proxy_send_telemetry";

pub trait SessionInfo: Send + Sync {
    /// Currently in use socket
    fn socket_addr(&self) -> SocketAddr;

    /// Current session state
    fn state(&self) -> &TdsSessionState;

    /// Mutate current session state
    fn set_state(&mut self, new_state: TdsSessionState);

    /// Session identifier
    fn session_id(&self) -> Ulid;

    /// Size of the TDS packet
    fn packet_size(&self) -> Arc<AtomicU16>;

    /// Username if SQL authentication is used
    fn get_sql_user_id(&self) -> Arc<str>;

    /// Username if SQL authentication is used
    fn set_sql_user_id(&mut self, sql_user_id: String);

    /// Session based database
    fn get_database(&self) -> Option<Arc<str>>;

    /// Set session based database
    fn set_database(&mut self, database: String);

    /// Schema to which connection is established
    fn get_schema(&self) -> Option<Arc<str>>;

    /// Set schema to which connection is established
    fn set_schema(&mut self, schema_name: String);

    /// TDS version of the communication
    fn tds_version(&self) -> Arc<str>;

    /// TDS server context
    fn tds_server_context(&self) -> Arc<ServerContext>;

    /// Counter of connection reset requests for this session
    fn connection_reset_request_count(&self) -> usize;

    /// Set client nonce for SQL authentication
    fn set_client_nonce(&mut self, nonce: [u8; 32]);

    /// Get client nonce for SQL authentication
    fn get_client_nonce(&self) -> Option<[u8; 32]>;

    /// Set server nonce for SQL authentication
    fn set_server_nonce(&mut self, nonce: [u8; 32]);

    /// Get server nonce for SQL authentication
    fn get_server_nonce(&self) -> Option<[u8; 32]>;

    /// Set session variable
    fn set_session_variable(&mut self, name: String, value: SessionVariable);

    /// Get session variable
    fn get_session_variable(&self, name: &str, expected: bool) -> &SessionVariable {
        if let Some(value) = self.get_session_variables().get(name) {
            return value;
        }
        if expected {
            self.report_value_not_set(name, &SessionVariable::None);
        }
        &SessionVariable::None
    }

    fn get_values_or_default<'a>(
        &self,
        default_values: &[&'a str],
        expected: bool,
    ) -> HashMap<&'a str, Arc<str>> {
        let mut values = HashMap::new();
        for v in default_values {
            let item = self.get_session_variable(v, expected);
            self.report_value_not_set(*v, item);
            values.insert(*v, item.get_value_or_default());
        }
        values
    }

    fn report_value_not_set(&self, name: &str, var: &SessionVariable) {
        if let SessionVariable::None = var {
            tracing::warn!(
                "Session variable '{}' not set for session with id {}",
                name,
                self.session_id()
            );
        }
    }

    /// Get all session variables
    fn get_session_variables(&self) -> HashMap<&str, &SessionVariable>;
}

pub enum SessionVariable {
    Some(Arc<str>),
    Default(Arc<str>),
    None,
}

impl SessionVariable {
    pub fn new(value: &str) -> Self {
        SessionVariable::Some(Arc::from(value))
    }

    pub fn new_default(value: &str) -> Self {
        SessionVariable::Default(Arc::from(value))
    }

    pub fn new_none() -> Self {
        SessionVariable::None
    }

    pub fn get_value_or_default(&self) -> Arc<str> {
        match self {
            SessionVariable::Some(value) => value.clone(),
            SessionVariable::Default(default_value) => default_value.clone(),
            SessionVariable::None => Arc::from(""),
        }
    }
}
