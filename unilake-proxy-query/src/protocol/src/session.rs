use crate::frontend::prot::{ServerInstance, TdsSessionState};
use crate::frontend::tds::server_context::ServerContext;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use ulid::Ulid;

pub const SESSION_VARIABLE_DIALECT: &str = "proxy_dialect";
pub const SESSION_VARIABLE_CATALOG: &str = "proxy_catalog";
pub const SESSION_VARIABLE_DATABASE: &str = "proxy_database";

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
    fn packet_size(&self) -> u16;

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
    fn get_session_variable(&self, name: &str) -> &SessionVariable {
        if let Some(value) = self.get_session_variables().get(name) {
            return value;
        }
        &SessionVariable::None
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
    pub fn get_value_or_default(&self) -> Arc<str> {
        match self {
            SessionVariable::Some(value) => value.clone(),
            SessionVariable::Default(default_value) => default_value.clone(),
            SessionVariable::None => Arc::from(""),
        }
    }
}

pub struct DefaultSession {
    socket_addr: SocketAddr,
    state: TdsSessionState,
    session_id: Ulid,
    packet_size: u16,
    sql_user_id: Option<Arc<str>>,
    database: Option<Arc<str>>,
    schema: Option<Arc<str>>,
    connection_reset_request_count: usize,
    tds_server_context: Arc<ServerContext>,
    client_nonce: Option<[u8; 32]>,
    server_nonce: Option<[u8; 32]>,
    session_variables: HashMap<String, SessionVariable>,
}

impl SessionInfo for DefaultSession {
    fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    fn state(&self) -> &TdsSessionState {
        &self.state
    }

    fn set_state(&mut self, new_state: TdsSessionState) {
        self.state = new_state
    }

    fn session_id(&self) -> Ulid {
        self.session_id
    }

    fn packet_size(&self) -> u16 {
        self.packet_size
    }

    fn get_sql_user_id(&self) -> Arc<str> {
        todo!()
    }

    fn set_sql_user_id(&mut self, sql_user_id: String) {
        self.sql_user_id = Some(Arc::from(sql_user_id));
    }

    fn get_database(&self) -> Option<Arc<str>> {
        if let Some(database) = &self.database {
            return Some(database.clone());
        }
        None
    }

    fn set_database(&mut self, catalog: String) {
        self.database = Some(Arc::from(catalog));
    }

    fn get_schema(&self) -> Option<Arc<str>> {
        if let Some(schema) = &self.schema {
            return Some(schema.clone());
        }
        None
    }

    fn set_schema(&mut self, db_name: String) {
        self.schema = Some(Arc::from(db_name));
    }

    fn tds_version(&self) -> Arc<str> {
        todo!()
    }

    fn tds_server_context(&self) -> Arc<ServerContext> {
        self.tds_server_context.clone()
    }

    fn connection_reset_request_count(&self) -> usize {
        todo!()
    }

    fn set_client_nonce(&mut self, nonce: [u8; 32]) {
        self.client_nonce = Some(nonce);
    }

    fn get_client_nonce(&self) -> Option<[u8; 32]> {
        self.client_nonce
    }

    fn set_server_nonce(&mut self, nonce: [u8; 32]) {
        self.server_nonce = Some(nonce);
    }

    fn get_server_nonce(&self) -> Option<[u8; 32]> {
        self.server_nonce
    }

    fn set_session_variable(&mut self, name: String, value: SessionVariable) {
        self.session_variables.insert(name, value);
    }

    fn get_session_variables(&self) -> HashMap<&str, &SessionVariable> {
        self.session_variables
            .iter()
            .map(|(k, v)| (k.as_ref(), v))
            .collect()
    }
}

impl DefaultSession {
    pub fn new(socket_addr: SocketAddr, instance: Arc<ServerInstance>) -> Self {
        DefaultSession {
            socket_addr,
            packet_size: instance.ctx.packet_size,
            session_id: instance.next_session_id(),
            sql_user_id: None,
            state: TdsSessionState::default(),
            database: None,
            schema: None,
            tds_server_context: instance.ctx.clone(),
            client_nonce: None,
            server_nonce: None,
            session_variables: DefaultSession::get_default_session_variable(),
            connection_reset_request_count: 0,
        }
    }

    fn get_default_session_variable() -> HashMap<String, SessionVariable> {
        let mut variables = HashMap::new();
        variables.insert(
            SESSION_VARIABLE_CATALOG.to_string(),
            SessionVariable::Default(Arc::from("main")),
        );
        variables.insert(
            SESSION_VARIABLE_DIALECT.to_string(),
            SessionVariable::Default(Arc::from("tsql")),
        );
        variables
    }
}
