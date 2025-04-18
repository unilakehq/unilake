use crate::backend::app::{
    FedResult, FedResultStream, FederatedFrontendHandler, FederatedRequestType,
};
use crate::backend::data::BackendInstance;
use crate::backend::starrocks::starrocks_session::StarRocksSession;
use crate::backend::telemetry::{QueryTelemetry, QueryTelemetryHandler};
use crate::frontend::tds::codec::token::{
    TokenColMetaData, TokenDone, TokenEnvChange, TokenError, TokenInfo, TokenLoginAck,
    TokenPreLoginFedAuthRequiredOption, TokenRow,
};
use crate::frontend::tds::codec::{
    BatchRequest, LoginMessage, OptionFlag2, PreloginMessage, RpcRequest, TdsBackendResponse,
};
use crate::frontend::tds::collation::Collation;
use crate::frontend::tds::prot::TdsWireHandlerFactory;
use crate::frontend::tds::server_context::ServerContext;
use crate::server_instance::ServerInstance;
use crate::sessions::{
    Session, SESSION_VARIABLE_CATALOG_NAME, SESSION_VARIABLE_DATABASE_NAME,
    SESSION_VARIABLE_DIALECT, SESSION_VARIABLE_SEND_TELEMETRY,
};
use async_trait::async_trait;
use futures::Sink;
use mysql_async::OptsBuilder;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use unilake_common::error::Result;
use unilake_common::error_code::ErrorCode;
use unilake_common::settings::settings_server_transparent_mode;
use unilake_security::handler::{HandleResult, SecurityHandler, SecurityHandlerError};
use unilake_security::repository::RepoRest;
use unilake_sql::{PolicyAccessRequestUrl, TranspilerDenyCause};

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
    ) -> Result<()>
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
        session: Arc<Session>,
        e: Error,
    ) -> Result<()>
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
    ) -> Result<()>
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
    ) -> Result<SecurityHandler> {
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
            Box::new(RepoRest::new(
                session_info.get_tenant_id().to_string(),
                instance.get_rest_client(),
            )),
            session_info.get_abac_model(),
        ))
    }

    async fn secure_query<C>(
        &self,
        client: &mut C,
        session_info: &StarRocksSession,
        query_telemetry: &mut QueryTelemetryHandler,
        query: &str,
    ) -> Result<Option<Arc<str>>>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        let mut start = std::time::Instant::now();
        let mut security_handler = self.get_new_security_handler(session_info).await?;
        let ulid = security_handler.get_query_id();
        query_telemetry.set_query_id(ulid.to_string());

        let values = session_info.get_values_or_default(
            &[
                SESSION_VARIABLE_DIALECT,
                SESSION_VARIABLE_CATALOG_NAME,
                SESSION_VARIABLE_DATABASE_NAME,
            ],
            true,
        );

        start = std::time::Instant::now();
        let query = security_handler
            .handle_query(
                query,
                values[SESSION_VARIABLE_DIALECT].as_ref(),
                values[SESSION_VARIABLE_CATALOG_NAME].as_ref(),
                values[SESSION_VARIABLE_DATABASE_NAME].as_ref(),
            )
            .await;
        tracing::trace!(
            "Elapsed time [SecurityHandler.handle_query]: {:?}",
            start.elapsed()
        );

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
        false
    }

    async fn handle_batch_request<C>(
        &self,
        client: &mut C,
        cancellation_token: CancellationToken,
        session: Arc<Session>,
        mut query_telemetry: QueryTelemetryHandler,
        query: &str,
    ) -> Result<()>
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
    ) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        if session
            .get_session_variable(SESSION_VARIABLE_SEND_TELEMETRY, false)
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
    ) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        // todo: we want to present the user with at least the access link to request for access, the other ones are errors and messages?
        // send request for url from the API
        // send back the results of this request in an error token
        let error_token = TokenError::new(
            0,
            0,
            0,
            "".to_string(),
            self.inner.server_instance.ctx.server_name.clone(),
            "".to_string(),
            0,
        );

        todo!()
    }

    async fn handle_error_result<C>(
        &self,
        client: &mut C,
        error: SecurityHandlerError,
    ) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        let mut error_token = TokenError::new(
            0,
            0,
            0,
            "".to_string(),
            self.inner.server_instance.ctx.server_name.clone(),
            "".to_string(),
            0,
        );

        // todo: properly handle these
        match error {
            SecurityHandlerError::Error(e) => {
                error_token.code = e.code() as u32;
                error_token.message = e.message();
                error_token.procedure = e.name()
            }
            SecurityHandlerError::ParserError(e, p) => {
                error_token.code = e.code() as u32;
                error_token.message = e.message();
                error_token.procedure = e.name()
            }
            SecurityHandlerError::SecurityError(e, s) => {
                error_token.code = e.code() as u32;
                error_token.message = e.message();
                error_token.procedure = e.name()
            }
        }

        self.send_token(client, error_token).await?;
        self.send_token(client, TokenDone::new_error(0)).await?;
        Ok(())
    }
}
#[async_trait]
impl TdsWireHandlerFactory for StarRocksTdsHandlerFactory {
    async fn open_session(&self, socket_addr: &SocketAddr) -> Result<Session> {
        tracing::info!("New session for: {}", socket_addr);
        todo!()
        // Ok(StarRocksSession::new(
        //     socket_addr.clone(),
        //     instance_info.clone(),
        //     None,
        //     None,
        // ))
    }

    async fn close_session(&self, session: Arc<Session>) {
        tracing::info!("Closing session for: {}", session.get_session_id());
        let instance = self.get_backend_instance(session).await;
        instance
            .remove_user_session(session.get_sql_user_id().to_string())
            .await;
        session.close().await;
    }

    async fn on_prelogin_request<C>(
        &self,
        client: &mut C,
        session: Arc<Session>,
        msg: &PreloginMessage,
    ) -> Result<()>
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
        // todo(mrhamburg): implement mars
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
        session: Arc<Session>,
        msg: &LoginMessage,
    ) -> Result<()>
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
            return Err(ErrorCode::TdsProtocol(
                "SSPI authentication is not supported",
            ));
        }

        // expect this to be basic auth, which will be implemented later
        // todo(mrhamburg): properly implement authentication
        if let Some(ref client_id) = msg.client_id {
            session.set_current_user_id(Arc::from(client_id.clone()));
        }

        // set database change
        let old_database = session.get_current_database().to_string();
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
        session.set_current_database(new_database);

        // set collation change
        self.send_token(
            client,
            TokenEnvChange::new_collation_change(None, Some(Collation::default())),
        )
        .await?;

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

    fn on_federated_authentication_token_message(&self, _session: Arc<Session>) {
        todo!()
    }

    async fn on_remote_procedure_call<C>(
        &self,
        client: &mut C,
        session: Arc<Session>,
        msg: &RpcRequest,
    ) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        if let Some(statement) = msg.parameters.first() {
            tracing::info!("Received RPC request: {:?}", statement.value);
        }

        // check for federated query
        let hash = msg.get_hash();
        if let Some(handler) =
            FederatedFrontendHandler::exec_request(hash, FederatedRequestType::Rpc(msg))?
        {
            return self.handle_fed_resultset(client, handler).await;
        }
        tracing::trace!("No federated query found for: {}", hash);

        Ok(())
    }

    async fn on_sql_batch_request<C>(
        &self,
        client: &mut C,
        session: Arc<Session>,
        msg: &BatchRequest,
    ) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        tracing::info!("Received SQL batch request: {}", &msg.query);

        // set query telemetry, for keeping track of query execution time
        let telemetry = QueryTelemetryHandler::new(self.inner.server_instance.clone());

        // check for federated query
        let hash = if let Ok(sent_hash) = u64::from_str(msg.query.as_str()) {
            sent_hash
        } else {
            msg.get_hash()
        };

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
                        .wait_timeout(Some(100))
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

    fn on_attention(&self, _session: Arc<Session>) {
        todo!()
    }
}
