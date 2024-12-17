mod extensions;
mod query;
mod session;

use crate::backend::app::generic::FedResult;
use crate::backend::app::{FedResultStream, FederatedFrontendHandler, FederatedRequestType};
use crate::backend::data::BackendInstance;
use crate::backend::starrocks::session::StarRocksSession;
use crate::backend::telemetry::{QueryTelemetry, QueryTelemetryHandler};
use crate::frontend::{
    prot::{
        ServerInstance, ServerInstanceMessage, SessionAuditMessage, SessionUserInfo,
        TdsWireHandlerFactory,
    },
    tds::server_context::ServerContext,
    BatchRequest, LoginMessage, OptionFlag2, PreloginMessage, TdsBackendResponse, TokenColMetaData,
    TokenDone, TokenEnvChange, TokenInfo, TokenLoginAck, TokenPreLoginFedAuthRequiredOption,
    TokenRow,
};
use crate::session::{
    SessionInfo, SESSION_VARIABLE_CATALOG, SESSION_VARIABLE_DATABASE, SESSION_VARIABLE_DIALECT,
    SESSION_VARIABLE_SEND_TELEMETRY,
};
use async_trait::async_trait;
use chrono::{DateTime, TimeDelta, Utc};
use futures::{Sink, StreamExt};
use mysql_async::{prelude::Queryable, Conn, Error, OptsBuilder, Pool};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use unilake_common::error::{TdsWireError, TdsWireResult, TokenError};
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
    pub async fn is_timed_out(&self) -> bool {
        let last_activity = self.last_activity_reported.lock().await;
        match last_activity.as_ref() {
            Some(last) => {
                Utc::now().signed_duration_since(*last)
                    > TimeDelta::minutes(self.activity_timeout_in_minutes as i64)
            }
            None => false,
        }
    }

    pub async fn get_conn(&self, userid: &str) -> TdsWireResult<Conn> {
        match self.mysql_pool.get_conn().await {
            Ok(conn) => {
                let mut session_counter = self.session_count.lock().await;
                if let Some(session_count) = session_counter.get_mut(userid) {
                    *session_count += 1;
                } else {
                    session_counter.insert(userid.to_string(), 1);
                }

                Ok(conn)
            }
            Err(_) => Err(TdsWireError::Protocol(
                "Failed to get connection from pool".to_string(),
            )),
        }
    }

    pub async fn drop_conn(&self, userid: &str) {
        let mut sessions = self.session_count.lock().await;
        if let Some(count) = sessions.get_mut(userid) {
            *count -= 1;
        }
    }

    async fn register_activity(&self) {
        if let Some(last_request) = *self.last_activity_reported.lock().await {
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
        *self.last_activity_reported.lock().await = Some(Utc::now());
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
            let mut backends = self.backends.write().await;
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

    pub async fn get_backend(
        &self,
        cluster_name: &str,
        register_activity: bool,
    ) -> Option<Arc<StarRocksBackend>> {
        let backend = self
            .backends
            .read()
            .await
            .get(cluster_name)
            .map(|x| x.clone());
        if register_activity && backend.is_some() {
            if let Some(ref backend) = backend {
                backend.register_activity().await
            }
        }
        backend
    }

    /// Send query and its handler to the audit system, the handler can obfuscate sensitive data
    /// and contains all information used in the transpiling process
    async fn audit_on_query<S: SessionInfo>(&self, user_info: &S, query: SecurityHandler) -> () {
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

    // async fn query_event(&self, query_id: Ulid, event_type: QueryEventType) {
    //     let time = std::time::SystemTime::now();
    //     // probably best to be implemented in the new query.rs environment
    //     // todo: implement this properly, send to a queue for further processing
    // }
}

pub struct StarRocksTdsHandlerFactory {
    inner: StarRocksTdsHandlerFactoryInnnerState,
}

impl StarRocksTdsHandlerFactory {
    pub fn new(server_instance: Arc<ServerInstance>) -> Self {
        StarRocksTdsHandlerFactory {
            inner: StarRocksTdsHandlerFactoryInnnerState::new(server_instance),
        }
    }

    async fn handle_frontend_error<C, TE>(
        &self,
        client: &mut C,
        session_info: &StarRocksSession,
        e: TE,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
        TE: Into<TokenError>,
    {
        //todo(mrhamburg): make sure this is also logged properly etc...
        let mut token = e.into();
        token.server = session_info.tds_server_context().server_name.clone();
        self.send_token(client, token).await?;
        self.send_token(client, TokenDone::new_error(0)).await?;
        Ok(())
    }

    async fn handle_backend_error<C>(
        &self,
        client: &mut C,
        session_info: &StarRocksSession,
        e: Error,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        //todo(mrhamburg): make sure this is also logged properly etc...
        let error_token = TokenError::new(
            0,
            0,
            0,
            e.to_string(),
            session_info.tds_server_context().server_name.clone(),
            "".to_string(),
            0,
        );
        self.send_token(client, error_token).await?;
        self.send_token(client, TokenDone::new_error(0)).await?;
        Ok(())
    }

    async fn handle_fed_resultset<C>(
        &self,
        client: &mut C,
        mut fed_result: FedResultStream,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
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

    async fn get_backend_instance(&self, session_info: &StarRocksSession) -> Arc<BackendInstance> {
        self.inner
            .server_instance
            .backend_handler
            .get_backend_instance(session_info.get_tenant_id().to_string())
            .await
    }

    async fn get_new_security_handler(
        &self,
        session_info: &StarRocksSession,
    ) -> TdsWireResult<SecurityHandler> {
        let instance = self.get_backend_instance(session_info).await;
        Ok(SecurityHandler::new(
            instance.get_cached_adapter(),
            session_info
                .get_session_model(
                    instance.get_ip_info_cache(),
                    instance.get_app_info_cache(),
                    instance.get_active_policy_id().await.unwrap_or(0),
                )
                .await?,
            instance
                .get_user_hit_rules(session_info.get_sql_user_id().to_string())
                .await,
            instance.get_cache_container(),
            Box::new(RepoRest::new(session_info.get_tenant_id().to_string())),
            session_info.get_abac_model(),
        ))
    }

    async fn secure_query<C>(
        &self,
        client: &mut C,
        session_info: &StarRocksSession,
        query_telemetry: &mut QueryTelemetryHandler,
        query: &str,
    ) -> TdsWireResult<Option<Arc<str>>>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        let mut security_handler = self.get_new_security_handler(session_info).await?;
        let ulid = security_handler.get_query_id();
        query_telemetry.set_query_id(ulid.to_string());

        let values = session_info.get_values_or_default(&[
            SESSION_VARIABLE_DIALECT,
            SESSION_VARIABLE_CATALOG,
            SESSION_VARIABLE_DATABASE,
        ]);

        let query = security_handler
            .handle_query(
                query,
                values[SESSION_VARIABLE_DIALECT].as_ref(),
                values[SESSION_VARIABLE_CATALOG].as_ref(),
                values[SESSION_VARIABLE_DATABASE].as_ref(),
            )
            .await;

        self.inner
            .audit_on_query(session_info, security_handler)
            .await;

        match query {
            Ok(q) => match q {
                HandleResult::Query(q) => Ok(Some(q)),
                HandleResult::AccessDenied(cause, access_links) => {
                    self.handle_telemetry_request(
                        client,
                        query_telemetry.end().await,
                        session_info,
                    )
                    .await?;
                    self.handle_access_denied_result(client, cause, access_links)
                        .await?;
                    Ok(None)
                }
            },
            Err(e) => {
                self.handle_telemetry_request(client, query_telemetry.end().await, session_info)
                    .await?;
                self.handle_error_result(client, e).await?;
                Ok(None)
            }
        }
    }

    /// Checks if transparent mode is enabled, used for debugging purposes
    fn get_transparent_mode_on() -> bool {
        if cfg!(debug_assertions) {
            return settings_server_transparent_mode();
        }
        // false
        true
    }

    async fn handle_batch_request<C>(
        &self,
        client: &mut C,
        cancellation_token: CancellationToken,
        session: &StarRocksSession,
        mut query_telemetry: QueryTelemetryHandler,
        query: &str,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        let mut conn = session.get_conn().await?;
        tracing::debug!("Connection id: {}", conn.id());

        // for debugging purposes we only secure the query if transparent mode is disabled
        let query = if Self::get_transparent_mode_on() {
            Arc::from(query)
        } else {
            match self
                .secure_query(client, session, &mut query_telemetry, query)
                .await?
            {
                None => return Ok(()),
                Some(q) => q,
            }
        };

        // todo(mrhamburg): handle query cancellation (either when dropping the connection or by sending an attention message to cancel)
        query_telemetry.start_backend_timer();
        let query_result = tokio::select! {
            result = conn.query_iter(query) => {
                query_telemetry.clock_backend_time();
                match result {
                    Ok(result) => {
                        Some(result)
                    }
                    Err(e) => {
                        self.handle_telemetry_request(client, query_telemetry.end().await, session).await?;
                        self.handle_backend_error(client, session, e).await?;
                        return Ok(())
                    }
                }
            },
            _ = cancellation_token.cancelled() => {
                eprintln!("Query was canceled.");
                None
            }
        };

        // send column metadata
        let mut result = query_result.unwrap();
        let mut columns = TokenColMetaData::new(result.columns_ref().len());
        for column in result.columns_ref() {
            columns.add_column(column);
        }

        // todo: add exclude time for send_token (telemetry), so we don't include network time
        self.send_token(client, columns).await?;

        // send rows
        let mut record_count = 0;
        let mut record_bytes = 0;
        while let Ok(Some(row)) = result.next().await {
            let token_row = TokenRow::from(row);
            record_count += 1;
            record_bytes += token_row.size_in_bytes();

            // todo: add exclude time for send_token (telemetry), so we don't include network time
            self.send_token(client, token_row).await?;
        }

        // set and send telemetry
        query_telemetry.set_processed_data(record_count, record_bytes as u64);
        self.handle_telemetry_request(client, query_telemetry.end().await, session)
            .await?;

        // send token done
        self.send_token(client, TokenDone::new_count(0, record_count))
            .await
    }

    async fn handle_telemetry_request<C>(
        &self,
        client: &mut C,
        telemetry: QueryTelemetry,
        session: &StarRocksSession,
    ) -> Result<(), TdsWireError>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        if session
            .get_session_variable(SESSION_VARIABLE_SEND_TELEMETRY)
            .get_value_or_default()
            .as_ref()
            != "true"
        {
            return self
                .send_token(
                    client,
                    telemetry.generate_telemetry_message_token(
                        self.inner.server_instance.ctx.clone().as_ref(),
                    ),
                )
                .await;
        }
        Ok(())
    }

    async fn handle_access_denied_result<C>(
        &self,
        client: &mut C,
        cause: Vec<TranspilerDenyCause>,
        access_links: Option<Vec<PolicyAccessRequestUrl>>,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        // todo: we want to present the user with at least the access link to request for access, the other ones are errors and messages?
        todo!()
    }

    async fn handle_error_result<C>(
        &self,
        client: &mut C,
        error: SecurityHandlerError,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        // todo: this could be an error like no user found
        todo!()
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
            None,
            None,
        ))
    }

    async fn close_session(&self, session: &mut StarRocksSession) {
        tracing::info!("Closing session for: {}", session.session_id());
        let instance = self.get_backend_instance(session).await;
        instance
            .remove_user_session(session.get_sql_user_id().to_string())
            .await;
        session.close().await;
    }

    async fn on_prelogin_request<C>(
        &self,
        client: &mut C,
        session_info: &mut StarRocksSession,
        msg: &PreloginMessage,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        let server_context = session_info.tds_server_context();
        let encryption = ServerContext::encryption_response(
            session_info.tds_server_context().as_ref(),
            msg.encryption,
        );

        let mut prelogin_msg = PreloginMessage::new();
        prelogin_msg.version = server_context.get_server_version();
        prelogin_msg.encryption = Some(encryption);
        prelogin_msg.mars = false;
        prelogin_msg.fed_auth_required = Some(false);
        prelogin_msg.instance_name = Some("".to_string());
        if let Some(nonce) = msg.nonce {
            session_info.set_client_nonce(nonce);
        }

        if server_context.fed_auth_options == TokenPreLoginFedAuthRequiredOption::FedAuthRequired {
            prelogin_msg.fed_auth_required = match msg.fed_auth_required {
                Some(a) => Some(a),
                None => None,
            };

            if msg.nonce.is_some() {
                prelogin_msg.nonce = Some(crate::frontend::utils::generate_random_nonce());
                session_info.set_server_nonce(prelogin_msg.nonce.unwrap());
            }
        }

        self.send_message(client, prelogin_msg).await
    }

    async fn on_login7_request<C>(
        &self,
        client: &mut C,
        session_info: &mut StarRocksSession,
        msg: &LoginMessage,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
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
            session_info.set_sql_user_id(client_id.clone());
        }

        // set database change
        let old_database = if let Some(old_database) = session_info.get_database() {
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
                &*session_info.tds_server_context(),
                5701,
                2,
                0,
                format!("Changed database context to '{}'", &new_database),
            ),
        )
        .await?;
        session_info.set_schema(new_database);

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
                &*session_info.tds_server_context(),
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
                &*session_info.tds_server_context(),
                5702,
                1,
                0,
                format!("Changed packet size to {}", "4096"),
            ),
        )
        .await?;

        // keep this information
        session_info.set_login_message(msg.clone());

        // create login ack token
        self.send_token(
            client,
            TokenLoginAck::new(session_info.tds_server_context()),
        )
        .await?;

        // check if session recovery is enabled
        if session_info.tds_server_context().session_recovery_enabled {
            // msg.add_token(FeatureAck::new_session_recovery());
        }

        self.send_token(client, TokenDone::new_final()).await
    }

    fn on_federated_authentication_token_message(&self, _session: &StarRocksSession) {
        todo!()
    }

    async fn on_sql_batch_request<C>(
        &self,
        client: &mut C,
        session_info: &mut StarRocksSession,
        msg: &BatchRequest,
    ) -> TdsWireResult<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        tracing::info!("Received SQL batch request: {}", &msg.query);

        // set query telemetry, for keeping track of query execution time
        let telemetry = QueryTelemetryHandler::new(self.inner.server_instance.clone());

        // check for federated query
        let hash = msg.get_hash();
        if let Some(handler) =
            FederatedFrontendHandler::exec_request(hash, FederatedRequestType::Query(msg))?
        {
            return self.handle_fed_resultset(client, handler).await;
        }
        tracing::trace!("No federated query found for: {}", hash);

        // handle initial session connection
        if !session_info.has_conn() {
            let backend = self
                .inner
                .get_or_add_backend("testing", || {
                    tracing::info!("Setting up StarRocks backend");
                    OptsBuilder::default()
                        .ip_or_hostname("10.255.255.17")
                        .tcp_port(9030)
                        .user(Some("root"))
                        .prefer_socket(Some(false))
                })
                .await;

            let conn = backend
                .get_conn(session_info.get_sql_user_id().as_ref())
                .await?;
            session_info.set_backend(backend.clone());
            session_info.set_conn(Mutex::new(conn));
        }

        // register activity to backend
        session_info.register_activity().await;

        // handle batch request
        let cancellation_token = CancellationToken::new();
        self.handle_batch_request(
            client,
            cancellation_token,
            session_info,
            telemetry,
            &msg.query,
        )
        .await
    }

    fn on_attention(&self, _session: &StarRocksSession) {
        todo!()
    }
}
