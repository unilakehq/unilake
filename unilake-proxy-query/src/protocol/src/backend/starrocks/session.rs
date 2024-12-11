use crate::backend::starrocks::StarRocksBackend;
use crate::frontend::prot::{ServerInstance, TdsSessionState};
use crate::frontend::tds::server_context::ServerContext;
use crate::session::{
    SessionInfo, SessionVariable, SESSION_VARIABLE_CATALOG, SESSION_VARIABLE_DATABASE,
    SESSION_VARIABLE_DIALECT, SESSION_VARIABLE_SECURITY_IMPERSONATE,
    SESSION_VARIABLE_SEND_TELEMETRY,
};
use casbin::{Cache, CoreApi, DefaultModel};
use chrono::Datelike;
use mysql_async::Conn;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU16;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use ulid::Ulid;
use unilake_common::error::{TdsWireError, TdsWireResult};
use unilake_common::model::{AppInfoModel, IpInfoModel, SessionModel};
use unilake_security::caching::layered_cache::MultiLayeredCache;
use unilake_security::HitRule;

pub struct StarRocksSession {
    socket_addr: SocketAddr,
    state: TdsSessionState,
    session_id: Ulid,
    packet_size: Arc<AtomicU16>,
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
    tenant_id: Arc<str>,
    workspace_id: Arc<str>,
    domain_id: Arc<str>,
    endpoint: Arc<str>,
    backend: Option<Arc<StarRocksBackend>>,
    conn: Option<Mutex<Conn>>,
    cached_rules: Option<Arc<Box<dyn Cache<u64, (String, HitRule)>>>>,
    server_instance: Arc<ServerInstance>,
}

impl StarRocksSession {
    pub fn new(
        socket_addr: SocketAddr,
        server_instance: Arc<ServerInstance>,
        conn: Option<Mutex<Conn>>,
        cached_rules: Option<Arc<Box<dyn Cache<u64, (String, HitRule)>>>>,
    ) -> Self {
        // let e = Arc::new(Mutex::new(
        //     CachedEnforcer::new(abac_model, adapter).await.unwrap(),
        // ));
        StarRocksSession {
            socket_addr,
            packet_size: Arc::new(AtomicU16::new(server_instance.ctx.packet_size)),
            session_id: server_instance.next_session_id(),
            sql_user_id: Some(Arc::from("fake_user_id")),
            state: TdsSessionState::default(),
            database: None,
            schema: None,
            tds_server_context: server_instance.ctx.clone(),
            client_nonce: None,
            server_nonce: None,
            session_variables: StarRocksSession::get_default_session_variable(),
            connection_reset_request_count: 0,
            branch_name: Arc::from(""),
            compute_id: Arc::from(""),
            tenant_id: Arc::from(""),
            workspace_id: Arc::from(""),
            domain_id: Arc::from(""),
            endpoint: Arc::from(""),
            conn,
            cached_rules,
            server_instance,
            backend: None,
        }
    }

    pub fn set_conn(&mut self, conn: Mutex<Conn>) {
        self.conn = Some(conn);
    }

    pub fn get_tenant_id(&self) -> Arc<str> {
        self.tenant_id.clone()
    }

    pub fn has_conn(&self) -> bool {
        self.conn.is_some()
    }

    pub async fn register_activity(&self) {
        if let Some(pool) = &self.backend {
            pool.register_activity().await;
        }
    }

    pub fn set_backend(&mut self, backend: Arc<StarRocksBackend>) {
        self.backend = Some(backend);
    }

    pub async fn get_conn(&self) -> TdsWireResult<MutexGuard<Conn>> {
        if let Some(conn) = &self.conn {
            return Ok(conn.lock().await);
        }
        Err(TdsWireError::Protocol(
            "No connection available".to_string(),
        ))
    }

    pub fn set_cached_rules(&mut self, cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>) {
        self.cached_rules = Some(cached_rules);
    }

    pub fn get_cached_rules(&self) -> Arc<Box<dyn Cache<u64, (String, HitRule)>>> {
        if let Some(cached_rules) = &self.cached_rules {
            return cached_rules.clone();
        }
        panic!("No cached rules available");
    }

    fn get_default_session_variable() -> HashMap<String, SessionVariable> {
        let mut variables = HashMap::new();
        variables.insert(
            SESSION_VARIABLE_CATALOG.to_string(),
            SessionVariable::new_default("default_catalog"),
        );
        variables.insert(
            SESSION_VARIABLE_DIALECT.to_string(),
            SessionVariable::new_default("tsql"),
        );
        variables.insert(
            SESSION_VARIABLE_DATABASE.to_string(),
            SessionVariable::new_default("default_schema"),
        );
        variables.insert(
            SESSION_VARIABLE_SEND_TELEMETRY.to_string(),
            SessionVariable::new_default("false"),
        );
        variables
    }

    pub async fn close(&self) {
        if let Some(pool) = &self.backend {
            if let Some(userid) = &self.sql_user_id {
                pool.drop_conn(userid.as_ref()).await;
            }
        }
    }

    pub async fn get_session_model(
        &self,
        ip_info: Arc<Box<MultiLayeredCache<String, IpInfoModel>>>,
        app_info: Arc<Box<MultiLayeredCache<String, AppInfoModel>>>,
        policy_id: u64,
    ) -> TdsWireResult<SessionModel> {
        // get connecting IP info, if available
        let ip_info = ip_info.get(&self.socket_addr.ip().to_string()).await;
        if ip_info.is_none() {
            tracing::error!("Failed to get IP info for {}", self.socket_addr);
            return Err(TdsWireError::Protocol("Failed to get IP info".to_string()));
        }
        let ip_info = ip_info.unwrap();

        // get connecting app info, if available
        let app_info = app_info.get(&"".to_string()).await;
        if app_info.is_none() {
            tracing::error!("Failed to get app info for {}", self.socket_addr);
            return Err(TdsWireError::Protocol("Failed to get app info".to_string()));
        }
        let app_info = app_info.unwrap();

        // get impersonate user id, if available
        let impersonate_user_id = if let SessionVariable::Some(v) =
            self.get_session_variable(SESSION_VARIABLE_SECURITY_IMPERSONATE)
        {
            Some(v.to_string())
        } else {
            None
        };

        // get current time and build session information
        let current_time = chrono::offset::Utc::now();
        Ok(SessionModel {
            impersonate_user_id,
            user_id: self
                .sql_user_id
                .as_ref()
                .clone()
                .expect("expecting user_id to exist after login")
                .to_string(),
            id: self.session_id.to_string(),
            app_id: app_info.app_id,
            app_name: app_info.app_name,
            app_type: app_info.app_type,
            app_driver: app_info.app_driver,
            source_ipv4: ip_info.ip_v4,
            country_iso2: ip_info.country_iso2,
            continent: ip_info.continent,
            timezone: ip_info.timezone,
            time: current_time.timestamp(),
            day_of_week: current_time.weekday().num_days_from_monday(),
            branch: self.branch_name.clone().to_string(),
            compute_id: self.compute_id.clone().to_string(),
            policy_id: policy_id.clone().to_string(),
            workspace_id: self.workspace_id.clone().to_string(),
            domain_id: self.domain_id.clone().to_string(),
        })
    }

    pub fn get_abac_model(&self) -> Option<DefaultModel> {
        self.server_instance.get_abac_model()
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

    fn packet_size(&self) -> Arc<AtomicU16> {
        self.packet_size.clone()
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
