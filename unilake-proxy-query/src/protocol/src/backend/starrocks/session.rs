use crate::frontend::prot::{ServerInstance, TdsSessionState};
use crate::frontend::tds::server_context::ServerContext;
use crate::security::context::ConnectionContext;
use crate::session::{
    SessionInfo, SessionVariable, SESSION_VARIABLE_CATALOG, SESSION_VARIABLE_DATABASE,
    SESSION_VARIABLE_DIALECT,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use ulid::Ulid;

pub struct StarRocksSession {
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
    branch_name: Arc<str>,
    compute_id: Arc<str>,
    endpoint: Arc<str>,
}

impl StarRocksSession {
    pub fn new(socket_addr: SocketAddr, instance: Arc<ServerInstance>) -> Self {
        StarRocksSession {
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
            session_variables: StarRocksSession::get_default_session_variable(),
            connection_reset_request_count: 0,
            branch_name: Arc::from(""),
            compute_id: Arc::from(""),
            endpoint: Arc::from(""),
        }
    }

    fn get_default_session_variable() -> HashMap<String, SessionVariable> {
        let mut variables = HashMap::new();
        variables.insert(
            SESSION_VARIABLE_CATALOG.to_string(),
            SessionVariable::Default(Arc::from("default_catalog")),
        );
        variables.insert(
            SESSION_VARIABLE_DIALECT.to_string(),
            SessionVariable::Default(Arc::from("tsql")),
        );
        variables
    }
}

impl StarRocksSession {}

impl SessionInfo for StarRocksSession {
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
        Arc::from("")
    }

    fn set_sql_user_id(&mut self, sql_user_id: String) {
        self.sql_user_id = Some(Arc::from(sql_user_id));
    }

    fn get_database(&self) -> Option<Arc<str>> {
        if let Some(db) = &self.database {
            return Some(db.clone());
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

impl From<&StarRocksSession> for ConnectionContext {
    fn from(value: &StarRocksSession) -> Self {
        ConnectionContext {
            branch_name: value
                .get_session_variable("branch_name")
                .get_value_or_default(),
            source_ip: Arc::from(value.socket_addr().ip().to_string()),
            compute_id: value
                .get_session_variable("compute_id")
                .get_value_or_default(),
            user_id: value.get_sql_user_id(),
            default_catalog: value
                .get_session_variable(SESSION_VARIABLE_CATALOG)
                .get_value_or_default(),
            default_database: value
                .get_session_variable(SESSION_VARIABLE_DATABASE)
                .get_value_or_default(),
            dialect: value
                .get_session_variable(SESSION_VARIABLE_DIALECT)
                .get_value_or_default(),
            role: None,
            session_id: Arc::from(value.session_id().to_string()),
            connection_timestamp: Arc::from(""),
            endpoint: value.endpoint.clone(),
            source_application: Arc::from("".to_string()),
            workspace_id: Arc::from("".to_string()),
        }
    }
}
