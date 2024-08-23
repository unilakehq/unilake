mod extensions;

use async_trait::async_trait;
use futures::Sink;
use mysql_async::{prelude::Queryable, OptsBuilder, Pool};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::frontend::{
    error::{TdsWireError, TdsWireResult},
    prot::{DefaultSession, ServerInstance, SessionInfo, TdsWireHandlerFactory},
    tds::server_context::ServerContext,
    BatchRequest, LoginMessage, OptionFlag2, PreloginMessage, TdsBackendResponse, TokenColMetaData,
    TokenDone, TokenEnvChange, TokenInfo, TokenLoginAck, TokenPreLoginFedAuthRequiredOption,
    TokenRow,
};

type StarRocksSession = DefaultSession;

struct StarRocksTdsHandlerFactoryInnnerState {
    pool: RwLock<Option<Arc<Pool>>>,
    // Pool is needed, functions to handle pool (add, get, disconnect and remove)
    // Backend actions are needed, handle a down cluster, spin up etc...
    // Probably also best to implement our own sessioninfo for starrocks for policy caching and things like that?
}

impl StarRocksTdsHandlerFactoryInnnerState {
    pub fn new() -> Self {
        Self {
            pool: RwLock::new(None),
        }
    }
    pub async fn set_pool(&self, pool: Pool) {
        let mut current_pool = self.pool.write().await;
        current_pool.replace(Arc::new(pool));
    }
    pub async fn get_pool(&self) -> Arc<Pool> {
        self.pool.read().await.clone().unwrap()
    }
}
pub struct StarRocksTdsHandlerFactory {
    inner: StarRocksTdsHandlerFactoryInnnerState,
}

impl StarRocksTdsHandlerFactory {
    pub fn new() -> Self {
        StarRocksTdsHandlerFactory {
            inner: StarRocksTdsHandlerFactoryInnnerState::new(),
        }
    }
}

#[async_trait]
impl TdsWireHandlerFactory<StarRocksSession> for StarRocksTdsHandlerFactory {
    fn open_session(
        &self,
        socket_addr: &SocketAddr,
        instance_info: Arc<ServerInstance>,
    ) -> Result<StarRocksSession, TdsWireError> {
        tracing::info!("New session for: {}", socket_addr);
        Ok(DefaultSession::new(
            socket_addr.clone(),
            instance_info.clone(),
        ))
    }

    fn close_session(&self, _session: &StarRocksSession) {
        todo!()
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
        let old_database = client.get_database().to_string();
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
        client.set_database(new_database);

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
        if self.inner.pool.read().await.is_none() {
            tracing::info!("Setting backend pool");
            let opts = OptsBuilder::default()
                .ip_or_hostname("10.255.255.17")
                .tcp_port(9030)
                .user(Some("root"))
                .prefer_socket(Some(false));
            self.inner.set_pool(Pool::new(opts)).await;
        }

        let pool = self.inner.get_pool().await;

        let mut conn = pool.get_conn().await.unwrap();
        tracing::info!("Connection pool id: {}", conn.id());

        let cancellation_token = CancellationToken::new();

        let query_result = tokio::select! {
            result = conn.query_iter(&msg.query) => {
                match result {
                    Ok(result) => {
                        Some(result)
                    }
                    Err(e) => {
                        eprintln!("Query failed: {}", e);
                        None
                    }
                }
            },
            _ = cancellation_token.cancelled() => {
                eprintln!("Query was canceled.");
                None
            }
        };
        let mut result = query_result.unwrap();

        let mut columns = TokenColMetaData::new();
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
