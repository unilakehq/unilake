// todo(mrhamburg): add feature to request for timings in the session (time in proxy, time in backend), this will also be needed for monitoring and troubleshooting
mod extensions;
mod session;

use crate::backend::app::generic::FedResult;
use crate::backend::app::{FedResultStream, FederatedFrontendHandler, FederatedRequestType};
use crate::backend::starrocks::session::StarRocksSession;
use crate::frontend::prot::{ServerInstanceMessage, SessionAuditMessage, SessionUserInfo};
use crate::frontend::{
    error::{TdsWireError, TdsWireResult},
    prot::{ServerInstance, TdsWireHandlerFactory},
    tds::server_context::ServerContext,
    BatchRequest, LoginMessage, OptionFlag2, PreloginMessage, TdsBackendResponse, TokenColMetaData,
    TokenDone, TokenEnvChange, TokenInfo, TokenLoginAck, TokenPreLoginFedAuthRequiredOption,
    TokenRow,
};
use crate::session::SessionInfo;
use async_trait::async_trait;
use chrono::{DateTime, TimeDelta, Utc};
use futures::{Sink, StreamExt};
use mysql_async::{prelude::Queryable, Conn, Error, OptsBuilder, Pool};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use ulid::Ulid;
use unilake_common::error::TokenError;
use unilake_security::handler::QueryHandler;

/// Acts like a wrapper around a mysql connection pool for StarRocks.
struct StarRocksPool {
    /// Connection pool for StarRocks
    /// todo(mrhamburg): we actually need multiple pools, for multiple FE nodes (so 3 FE nodes, is 3 pools and load-balance connections)?
    pool: Pool,
    bound_sessions: RwLock<HashMap<Ulid, Arc<RwLock<Conn>>>>,
    last_connection_request: Mutex<Option<DateTime<Utc>>>,
    activity_timeout_in_minutes: u16,
}

impl StarRocksPool {
    async fn update_last_connection_request(&self) {
        let mut current = self.last_connection_request.lock().await;
        *current = Some(Utc::now());
    }

    /// Checks if the current connection pool has not been used and has timed out. If so, the connection pool can be removed and the backend instance can be shutdown.
    pub async fn is_timed_out(&self) -> bool {
        let last_request = self.last_connection_request.lock().await;
        match last_request.as_ref() {
            Some(last) => {
                Utc::now().signed_duration_since(*last)
                    > TimeDelta::minutes(self.activity_timeout_in_minutes as i64)
            }
            None => false,
        }
    }

    pub async fn get_conn(&self, session_id: Ulid) -> Arc<RwLock<Conn>> {
        self.update_last_connection_request().await;
        if let Some(conn) = self.bound_sessions.read().await.get(&session_id) {
            return conn.clone();
        }

        let conn = self.pool.get_conn().await;
        if let Err(err) = conn {
            match err {
                Error::Io(io_err) => {
                    eprintln!("Failed to get connection from pool: {}", io_err);
                    eprintln!("This is a connection-related error.");
                }
                _ => {
                    eprintln!("Failed to get connection from pool: {}", err);
                    eprintln!("This is not a connection-related error.");
                }
            }
        } else {
            {
                let mut bound_sessions = self.bound_sessions.write().await;
                bound_sessions.insert(session_id, Arc::new(RwLock::new(conn.unwrap())));
            }
            if let Some(conn) = self.bound_sessions.read().await.get(&session_id) {
                return conn.clone();
            }
        }
        todo!()
    }

    pub async fn release_conn(&self, session_id: Ulid) {
        {
            if let Some(conn) = self.bound_sessions.read().await.get(&session_id) {
                let mut conn = conn.write().await;
                let _ = conn.reset().await;
            }
        }
        self.bound_sessions.write().await.remove(&session_id);
    }
}

struct StarRocksTdsHandlerFactoryInnnerState {
    pools: RwLock<HashMap<String, Arc<StarRocksPool>>>,
    server_instance: Arc<ServerInstance>,
    last_activity_reported: Mutex<Option<DateTime<Utc>>>,
    // Pool is needed, functions to handle pool (add, get, disconnect and remove)
    // Backend actions are needed, handle a down cluster, spin up etc...
    // Probably also best to implement our own sessioninfo for starrocks for policy caching and things like that?
}

impl StarRocksTdsHandlerFactoryInnnerState {
    pub fn new(server_instance: Arc<ServerInstance>) -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            server_instance,
            last_activity_reported: Mutex::new(None),
        }
    }

    pub async fn get_or_add_pool<F>(&self, cluster_name: &str, f: F) -> Arc<StarRocksPool>
    where
        F: FnOnce() -> OptsBuilder,
    {
        {
            let found = self.get_pool(cluster_name, true).await;
            if let Some(pool) = found {
                return pool.clone();
            }
        }
        {
            let mut pools = self.pools.write().await;
            let opts = f();
            let pool = Pool::new(opts);

            // todo(mrhamburg): also requires pooloptions and constraints (min max pool size for example)
            pools.insert(
                cluster_name.to_string(),
                Arc::new(StarRocksPool {
                    pool,
                    last_connection_request: Mutex::new(None),
                    //todo(mrhamburg): determine this, don't think 60 minutes is a good fit
                    activity_timeout_in_minutes: 60, // Default to 60 minutes
                    bound_sessions: RwLock::new(HashMap::new()),
                }),
            );
        }

        self.get_pool(cluster_name, false).await.unwrap()
    }

    pub async fn get_pool(
        &self,
        cluster_name: &str,
        register_activity: bool,
    ) -> Option<Arc<StarRocksPool>> {
        if register_activity {
            self.pool_register_connection_activity(cluster_name).await;
        }
        self.pools.read().await.get(cluster_name).map(|x| x.clone())
    }

    async fn pool_register_connection_activity(&self, pool_name: &str) {
        // only every 15 seconds
        // todo(mrhamburg): make sure this 15 seconds interval is configurable
        if let Some(last_request) = *self.last_activity_reported.lock().await {
            let elapsed = Utc::now().signed_duration_since(last_request);
            if elapsed < TimeDelta::seconds(15) {
                return;
            }
        }

        let result =
            self.server_instance
                .process_message(ServerInstanceMessage::ActivityConnection(
                    pool_name.to_string(),
                ));

        if let Err(err) = result {
            tracing::error!("Failed to register connection activity: {}", err);
        }
        *self.last_activity_reported.lock().await = Some(Utc::now());
    }

    async fn audit_on_query<S: SessionInfo>(&self, user_info: &S, query: QueryHandler) -> () {
        if let Err(e) = self
            .server_instance
            .process_message(ServerInstanceMessage::Audit(SessionAuditMessage::SqlQuery(
                SessionUserInfo::from(user_info),
                query,
            )))
        {
            //todo(mrhamburg): log this properly
            todo!()
        }
    }
}

pub struct StarRocksTdsHandlerFactory {
    inner: StarRocksTdsHandlerFactoryInnnerState,
}

// impl From<ParserError> for TokenError {
//     fn from(value: ParserError) -> Self {
//         if let Some(value) = value.errors.first() {
//             TokenError {
//                 line: value.line as u32,
//                 code: 0,
//                 message: value.description.to_string(),
//                 class: 0,
//                 procedure: "".to_string(),
//                 server: "".to_string(),
//                 state: 0,
//             }
//         } else {
//             TokenError {
//                 code: 0,
//                 state: 0,
//                 class: 0,
//                 message: value.message,
//                 server: "".to_string(),
//                 procedure: "".to_string(),
//                 line: 0,
//             }
//         }
//     }
// }

impl StarRocksTdsHandlerFactory {
    pub fn new(server_instance: Arc<ServerInstance>) -> Self {
        StarRocksTdsHandlerFactory {
            inner: StarRocksTdsHandlerFactoryInnnerState::new(server_instance),
        }
    }
    async fn handle_backend_error<C>(&self, client: &mut C, e: Error) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send + SessionInfo,
    {
        //todo(mrhamburg): make sure this is also logged properly etc...
        let error_token = TokenError::new(
            0,
            0,
            0,
            e.to_string(),
            client.tds_server_context().server_name.clone(),
            "".to_string(),
            0,
        );
        // todo(mrhamburg): fix this
        // self.send_token(client, error_token).await?;
        self.send_token(client, TokenDone::new_error(0)).await?;
        Ok(())
    }

    async fn handle_fed_resultset<C>(
        &self,
        client: &mut C,
        mut fed_result: FedResultStream,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send + SessionInfo,
    {
        while let Some(result) = fed_result.next().await {
            match result {
                Ok(fed_result) => match fed_result {
                    FedResult::Tabular(mut result) => {
                        self.send_token(client, TokenColMetaData::from(&mut result))
                            .await?;
                        let mut count = 0;
                        for row in result {
                            self.send_token(client, row).await?;
                            count += 1;
                        }
                        let token_done = TokenDone::new_count(0, count);
                        self.send_token(client, token_done).await?;
                    }
                    FedResult::Info(_) => todo!(),
                    FedResult::State(_) => todo!(),
                    FedResult::Empty => self.send_token(client, TokenDone::new_count(0, 0)).await?,
                },
                Err(_) => todo!(),
            }
        }
        Ok(())
    }
}

#[async_trait]
impl TdsWireHandlerFactory<StarRocksSession> for StarRocksTdsHandlerFactory {
    async fn open_session(
        &self,
        socket_addr: &SocketAddr,
        instance_info: Arc<ServerInstance>,
    ) -> Result<StarRocksSession, TdsWireError> {
        tracing::info!("New session for: {}", socket_addr);
        Ok(StarRocksSession::new(
            socket_addr.clone(),
            instance_info.clone(),
        ))
    }

    async fn close_session(&self, session: &StarRocksSession) {
        tracing::info!("Closing session for: {}", session.session_id());
        if let Some(pool) = self.inner.get_pool("testing", false).await {
            pool.release_conn(session.session_id()).await;
        }
    }

    async fn on_prelogin_request<C>(
        &self,
        client: &mut C,
        msg: &PreloginMessage,
    ) -> TdsWireResult<()>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
    {
        let server_context = client.tds_server_context();
        let encryption = ServerContext::encryption_response(
            client.tds_server_context().as_ref(),
            msg.encryption,
        );

        let mut prelogin_msg = PreloginMessage::new();
        prelogin_msg.version = server_context.get_server_version();
        prelogin_msg.encryption = Some(encryption);
        prelogin_msg.mars = false;
        prelogin_msg.fed_auth_required = Some(false);
        prelogin_msg.instance_name = Some("".to_string());
        if let Some(nonce) = msg.nonce {
            client.set_client_nonce(nonce);
        }

        if server_context.fed_auth_options == TokenPreLoginFedAuthRequiredOption::FedAuthRequired {
            prelogin_msg.fed_auth_required = match msg.fed_auth_required {
                Some(a) => Some(a),
                None => None,
            };

            if msg.nonce.is_some() {
                prelogin_msg.nonce = Some(crate::frontend::utils::generate_random_nonce());
                client.set_server_nonce(prelogin_msg.nonce.unwrap());
            }
        }

        self.send_message(client, prelogin_msg).await
    }

    async fn on_login7_request<C>(&self, client: &mut C, msg: &LoginMessage) -> TdsWireResult<()>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
    {
        if let Some(ref dbname) = msg.db_name {
            tracing::info!("Login request for database: {}", dbname);
        }

        // todo(mrhamburg): check for tds version

        // check for fed auth
        if let Some(ref _fed_auth_ext) = msg.fed_auth_ext {
            todo!()
        }

        // check for sspi (which we do not support)
        if msg.option_flags_2.contains(OptionFlag2::IntegratedSecurity) {
            return Err(TdsWireError::Protocol(
                "SSPI authentication is not supported".to_string(),
            ));
        }

        // expect this to be basic auth, which will be implemented later
        // todo(mrhamburg): implement authentication
        if let Some(ref client_id) = msg.client_id {
            client.set_sql_user_id(client_id.clone());
        }

        // set database change
        let old_database = if let Some(old_database) = client.get_database() {
            old_database.clone().to_string()
        } else {
            "".to_string()
        };
        let new_database = msg.db_name.clone().unwrap_or_else(|| "main".to_string());
        self.send_token(
            client,
            TokenEnvChange::new_database_change(old_database, new_database.clone()),
        )
        .await?;
        self.send_token(
            client,
            TokenInfo::new(
                &*client.tds_server_context(),
                5701,
                2,
                0,
                format!("Changed database context to '{}'", &new_database),
            ),
        )
        .await?;
        client.set_schema(new_database);

        // set collation change
        // return_msg.add_token(TokenEnvChange::new_collation_change(
        //     "".to_string(),
        //     "".to_string(),
        // ));

        // set language change
        self.send_token(
            client,
            TokenEnvChange::new_language_change("".to_string(), "us_english".to_string()),
        )
        .await?;
        self.send_token(
            client,
            TokenInfo::new(
                &*client.tds_server_context(),
                5703,
                1,
                0,
                format!("Changed language to '{}'", "us_english"),
            ),
        )
        .await?;

        // set packet size change
        self.send_token(
            client,
            TokenEnvChange::new_packet_size_change("4096".to_string(), "4096".to_string()),
        )
        .await?;
        self.send_token(
            client,
            TokenInfo::new(
                &*client.tds_server_context(),
                5702,
                1,
                0,
                format!("Changed packet size to {}", "4096"),
            ),
        )
        .await?;

        // create login ack token
        self.send_token(client, TokenLoginAck::new(client.tds_server_context()))
            .await?;

        // check if session recovery is enabled
        if client.tds_server_context().session_recovery_enabled {
            // msg.add_token(FeatureAck::new_session_recovery());
        }

        self.send_token(client, TokenDone::new_final()).await
    }

    fn on_federated_authentication_token_message(&self, _session: &StarRocksSession) {
        todo!()
    }

    async fn on_sql_batch_request<C>(&self, client: &mut C, msg: &BatchRequest) -> TdsWireResult<()>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
    {
        tracing::info!("Received SQL batch request: {}", &msg.query);
        // check for federated query
        let hash = msg.get_hash();
        if let Some(handler) =
            FederatedFrontendHandler::exec_request(hash, FederatedRequestType::Query(msg))?
        {
            return self.handle_fed_resultset(client, handler).await;
        }
        tracing::trace!("No federated query found for: {}", hash);

        let pool = self
            .inner
            .get_or_add_pool("testing", || {
                tracing::info!("Setting backend pool");
                OptsBuilder::default()
                    .ip_or_hostname("10.255.255.17")
                    .tcp_port(9030)
                    .user(Some("root"))
                    .prefer_socket(Some(false))
            })
            .await;

        let conn = pool.get_conn(client.session_id()).await;
        let mut conn = conn.write().await;
        tracing::debug!("Connection id: {}", conn.id());

        // todo(mrhamburg): handle query cancellation (either when dropping the connection or by sending an attention message to cancel)
        let cancellation_token = CancellationToken::new();
        let mut query_handler = QueryHandler::new();
        let query = query_handler
            // todo(mrhamburg): properly bring back dialect, catalog and database here
            .handle_query(&msg.query, "", "", "")
            .ok()
            // todo(mrhamburg): implement error handling, remove unwraps or oks
            .unwrap()
            .to_string();

        // send query and its handler to the audit system, the handler can obfuscate sensitive data
        // and contains all information used in the transpiling process
        self.inner.audit_on_query(client, query_handler).await;

        let query_result = tokio::select! {
            result = conn.query_iter(query) => {
                match result {
                    Ok(result) => {
                        Some(result)
                    }
                    Err(e) => {
                        self.handle_backend_error(client, e).await?;
                        return Ok(())
                    }
                }
            },
            _ = cancellation_token.cancelled() => {
                eprintln!("Query was canceled.");
                None
            }
        };
        let mut result = query_result.unwrap();
        let mut columns = TokenColMetaData::new(result.columns_ref().len());
        for column in result.columns_ref() {
            columns.add_column(column);
        }
        self.send_token(client, columns).await?;

        let mut count = 0;
        while let Ok(Some(row)) = result.next().await {
            let token_row = TokenRow::from(row);
            self.send_token(client, token_row).await?;
            count += 1;
        }

        self.send_token(client, TokenDone::new_count(0, count))
            .await
    }

    fn on_attention(&self, _session: &StarRocksSession) {
        todo!()
    }
}
