// todo: add implementation of backend handling here (sessions, pools, backend management)

use crate::backend::app::{FedResult, FedResultStream};
use crate::backend::backend_data::BackendData;
use crate::backend::starrocks::starrocks_session::StarRocksSession;
use crate::backend::telemetry::{QueryTelemetry, QueryTelemetryHandler};
use crate::frontend::tds::codec::token::{TokenColMetaData, TokenDone, TokenError, TokenRow};
use crate::frontend::tds::codec::TdsBackendResponse;
use crate::server_instance::ServerInstance;
use crate::session::{ServerInstanceMessage, SessionAuditMessage, SessionInfo, SessionUserInfoEto};
use crate::sessions::{
    Session, SESSION_VARIABLE_CATALOG_NAME, SESSION_VARIABLE_DATABASE_NAME,
    SESSION_VARIABLE_DIALECT, SESSION_VARIABLE_SEND_TELEMETRY,
};
use chrono::{DateTime, TimeDelta, Utc};
use futures::Sink;
use mysql_async::{Conn, OptsBuilder, Pool};
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use unilake_common::error::Result;
use unilake_common::error_code::ErrorCode;
use unilake_common::settings::{
    settings_backend_register_activity_timeout_in_seconds, settings_server_transparent_mode,
};
use unilake_security::handler::{HandleResult, SecurityHandler, SecurityHandlerError};
use unilake_security::repository::RepoRest;
use unilake_sql::{PolicyAccessRequestUrl, TranspilerDenyCause};

pub(crate) struct StarRocksBackend {
    /// todo(mrhamburg): we actually need multiple pools, for multiple FE nodes (so 3 FE nodes, is 3 pools and load-balance connections)?
    cluster_id: String,
    mysql_pool: Pool,
    last_activity_reported: Mutex<Option<DateTime<Utc>>>,
    activity_timeout_in_minutes: u16,
    server_instance: Arc<ServerInstance>,
    session_count: Mutex<HashMap<String, u64>>,
    // todo: multiple cache instances are needed here for model information, and we need a single server instance based adapter for loading policy files
    // todo: the above also requires a handler for cache changes (redis mq/kafka) -> backend will handle this.
}

impl StarRocksBackend {
    /// Checks if the current connection pool has not been used and has timed out. If so, the connection pool can be removed and the backend instance can be shutdown.
    // todo(mrhamburg): determine if this is necessary or if we can remove it entirely
    pub fn is_timed_out(&self) -> bool {
        let last_activity = self.last_activity_reported.lock();
        match last_activity.as_ref() {
            Some(last) => {
                Utc::now().signed_duration_since(*last)
                    > TimeDelta::minutes(self.activity_timeout_in_minutes as i64)
            }
            None => false,
        }
    }

    pub fn get_conn(&self, userid: &str) -> Result<Conn> {
        match self.mysql_pool.get_conn() {
            Ok(conn) => {
                let mut session_counter = self.session_count.lock();
                if let Some(session_count) = session_counter.get_mut(userid) {
                    *session_count += 1;
                } else {
                    session_counter.insert(userid.to_string(), 1);
                }

                Ok(conn)
            }
            Err(_) => Err(ErrorCode::StarRocksConnectionPoolError(
                "Failed to get connection from pool",
            )),
        }
    }

    pub fn drop_conn(&self, userid: &str) {
        let mut sessions = self.session_count.lock();
        if let Some(count) = sessions.get_mut(userid) {
            *count -= 1;
        }
    }

    fn register_activity(&self) {
        if let Some(last_request) = *self.last_activity_reported.lock() {
            let timeout = settings_backend_register_activity_timeout_in_seconds();
            let elapsed = Utc::now().signed_duration_since(last_request);
            if elapsed < TimeDelta::seconds(timeout) {
                return;
            }
        }

        let result =
            self.server_instance
                .process_message(ServerInstanceMessage::ActivityConnection(
                    self.cluster_id.to_string(),
                ));

        if let Err(err) = result {
            tracing::error!("Failed to register connection activity: {}", err);
        }
        *self.last_activity_reported.lock() = Some(Utc::now());
    }
}

struct StarRocksTdsHandlerFactoryInnnerState {
    backends: RwLock<HashMap<String, Arc<StarRocksBackend>>>,
    server_instance: Arc<ServerInstance>,
    // Pool is needed, functions to handle pool (add, get, disconnect and remove)
    // Backend actions are needed, handle a down cluster, spin up etc...
    // Probably also best to implement our own sessioninfo for starrocks for policy caching and things like that?
}

impl StarRocksTdsHandlerFactoryInnnerState {
    pub fn new(server_instance: Arc<ServerInstance>) -> Self {
        Self {
            backends: RwLock::new(HashMap::new()),
            server_instance,
        }
    }

    pub async fn get_or_add_backend<F>(&self, cluster_id: &str, f: F) -> Arc<StarRocksBackend>
    where
        F: FnOnce() -> OptsBuilder,
    {
        {
            let found = self.get_backend(cluster_id, true).await;
            if let Some(backend) = found {
                return backend.clone();
            }
        }
        {
            let mut backends = self.backends.write();
            let opts = f();
            let pool = Pool::new(opts);

            // todo(mrhamburg): also requires pooloptions and constraints (min max pool size for example)
            backends.insert(
                cluster_id.to_string(),
                Arc::new(StarRocksBackend {
                    cluster_id: cluster_id.to_string(),
                    mysql_pool: pool,
                    last_activity_reported: Mutex::new(None),
                    //todo(mrhamburg): determine this, don't think 60 minutes is a good fit, should be configurable (global config)
                    activity_timeout_in_minutes: 60, // Default to 60 minutes
                    server_instance: self.server_instance.clone(),
                    session_count: Mutex::new(HashMap::new()),
                }),
            );
        }

        self.get_backend(cluster_id, false).await.unwrap()
    }

    pub fn get_backend(
        &self,
        cluster_name: &str,
        register_activity: bool,
    ) -> Option<Arc<StarRocksBackend>> {
        let backend = self.backends.read().get(cluster_name).map(|x| x.clone());
        if register_activity && backend.is_some() {
            if let Some(ref backend) = backend {
                backend.register_activity()
            }
        }
        backend
    }

    /// Send query and its handler to the audit system, the handler can obfuscate sensitive data
    /// and contains all information used in the transpiling process
    async fn audit_on_query<S: SessionInfo>(&self, user_info: &S, query: SecurityHandler) -> () {
        tracing::debug!("Executing query on backend: {:?}", query.get_output_query());
        if let Err(e) = self
            .server_instance
            .process_message(ServerInstanceMessage::Audit(SessionAuditMessage::SqlQuery(
                SessionUserInfoEto::from(user_info),
                query,
            )))
        {
            tracing::error!("Failed to send audit query: {}", e);
        }
    }

    // async fn query_event(&self, query_id: Ulid, event_type: QueryEventType) {
    //     let time = std::time::SystemTime::now();
    //     // probably best to be implemented in the new query.rs environment
    //     // todo: implement this properly, send to a queue for further processing
    // }
}
