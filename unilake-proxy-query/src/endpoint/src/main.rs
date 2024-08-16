use async_trait::async_trait;
use futures::{Sink, SinkExt};
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use unilake_wire_frontend::codec::process_socket;
use unilake_wire_frontend::error::{TdsWireError, TdsWireResult};
use unilake_wire_frontend::prot::{DefaultSession, ServerInstance, SessionInfo, TdsWireHandler};
use unilake_wire_frontend::tds::codec::{
    BatchRequest, DoneStatus, EnvChangeType, FeatureAck, FixedLenType, LoginMessage, OptionFlag2,
    PacketType, PreloginMessage, ResponseMessage, TdsBackendResponse, TdsBackendResponseHandler,
    TdsMessage, TdsToken, TokenColMetaData, TokenDone, TokenEnvChange, TokenInfo, TokenLoginAck,
    TokenPreLoginFedAuthRequiredOption, TokenRow, TypeInfo,
};
use unilake_wire_frontend::tds::server_context::ServerContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:1433".to_string());

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    let factory = Arc::new(DefaultTdsWireHandler {});
    // todo(mrhamburg): use bgworker for graceful shutdown
    let (instance, _) = {
        let instance = ServerInstance::new(ServerContext::default());
        let (instance, bgworker) = instance.start_instance();
        (instance, bgworker)
    };

    loop {
        let (socket, _) = listener.accept().await?;
        let factory = factory.clone();
        let instance = instance.clone();

        tokio::spawn(async move { process_socket(socket, None, factory, instance).await });
    }
}

struct DefaultTdsWireHandler {}
#[async_trait]
impl TdsWireHandler<DefaultSession> for DefaultTdsWireHandler {
    /// Create a new TDS server session
    fn open_session(
        &self,
        socket_addr: &std::net::SocketAddr,
        instance_info: Arc<ServerInstance>,
    ) -> Result<DefaultSession, TdsWireError> {
        todo!()
    }

    /// Close TDS server session
    fn close_session(&self, session: &DefaultSession) {
        todo!()
    }

    /// Called when pre-login request arrives
    async fn on_prelogin_request(
        &self,
        session_info: &mut DefaultSession,
        response_hanlder: &mut TdsBackendResponseHandler<
            '_,
            impl Sink<TdsBackendResponse, Error = TdsWireError> + Unpin,
        >,
        msg: &PreloginMessage,
    ) -> TdsWireResult<()> {
        todo!()
    }

    /// Called when login request arrives, returns true if authentication is successful
    async fn on_login7_request(
        &self,
        session_info: &mut DefaultSession,
        response_hanlder: &mut TdsBackendResponseHandler<
            '_,
            impl Sink<TdsBackendResponse, Error = TdsWireError> + Unpin,
        >,
        msg: &LoginMessage,
    ) -> TdsWireResult<bool> {
        todo!()
    }

    /// Called when federated authentication token message arrives. Called only when
    /// such a message arrives in response to federated authentication info, not when the
    /// token is part of a login request.
    fn on_federated_authentication_token_message(&self, session: &mut DefaultSession) {}

    /// Called when SQL batch request arrives
    async fn on_sql_batch_request(
        &self,
        session_info: &mut DefaultSession,
        response_hanlder: &mut TdsBackendResponseHandler<
            '_,
            impl Sink<TdsBackendResponse, Error = TdsWireError> + Unpin + Send,
        >,
        msg: &BatchRequest,
    ) -> TdsWireResult<()> {
        todo!()
    }

    /// Called when attention arrives
    fn on_attention(&self, session: &mut DefaultSession) {
        todo!()
    }
}

// #[allow(unused_variables)]
// #[async_trait]
// impl TdsWireHandler for Something_old {
//     fn open_session(
//         &self,
//         socket_addr: &std::net::SocketAddr,
//         instance_info: Arc<ServerInstance>,
//     ) -> Result<unilake_wire_frontend::prot::DefaultSession, TdsWireError> {
//         tracing::info!("New session for: {}", socket_addr);
//         Ok(DefaultSession::new(
//             socket_addr.clone(),
//             instance_info.clone(),
//         ))
//     }

//     fn close_session(&self, session: &unilake_wire_frontend::prot::DefaultSession) {
//         todo!()
//     }

//     async fn on_prelogin_request<C>(
//         &self,
//         client: &mut C,
//         msg: &PreloginMessage,
//     ) -> TdsWireResult<()>
//     where
//         C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
//     {
//         let server_context = client.tds_server_context();
//         let encryption = ServerContext::encryption_response(
//             client.tds_server_context().as_ref(),
//             msg.encryption,
//         );

//         let mut prelogin_msg = PreloginMessage::new();
//         prelogin_msg.version = server_context.get_server_version();
//         prelogin_msg.encryption = Some(encryption);
//         prelogin_msg.mars = false;
//         prelogin_msg.fed_auth_required = Some(false);
//         prelogin_msg.instance_name = Some("".to_string());
//         if let Some(nonce) = msg.nonce {
//             client.set_client_nonce(nonce);
//         }

//         if server_context.fed_auth_options == TokenPreLoginFedAuthRequiredOption::FedAuthRequired {
//             prelogin_msg.fed_auth_required = match msg.fed_auth_required {
//                 Some(a) => Some(a),
//                 None => None,
//             };

//             if msg.nonce.is_some() {
//                 prelogin_msg.nonce = Some(unilake_wire_frontend::utils::generate_random_nonce());
//                 client.set_server_nonce(prelogin_msg.nonce.unwrap());
//             }
//         }
//         // let mut response_handler = TdsBackendResponseHandler::new();
//         // response_handler.add_message(prelogin_msg);
//         // client.feed(response_handler).await;

//         Ok(())
//     }

//     async fn on_login7_request<C>(&self, client: &mut C, msg: &LoginMessage) -> TdsWireResult<bool>
//     where
//         C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
//     {
//         if let Some(ref dbname) = msg.db_name {
//             tracing::info!("Login request for database: {}", dbname);
//         }

//         // todo(mrhamburg): check for tds version

//         // check for fed auth
//         if let Some(ref fed_auth_ext) = msg.fed_auth_ext {
//             todo!()
//         }

//         // check for sspi (which we do not support)
//         if msg.option_flags_2.contains(OptionFlag2::IntegratedSecurity) {
//             return Err(TdsWireError::Protocol(
//                 "SSPI authentication is not supported".to_string(),
//             ));
//         }

//         // expect this to be basic auth, which will be implemented later
//         // todo(mrhamburg): implement authentication
//         if let Some(ref client_id) = msg.client_id {
//             client.set_sql_user_id(client_id.clone());
//         }

//         // Create return message
//         // let mut return_handler = TdsBackendResponseHandler::new();
//         let mut return_handler = TdsBackendResponse::new(90, 9);

//         // set database change
//         let old_database = client.get_database().to_string();
//         let new_database = msg.db_name.clone().unwrap_or_else(|| "main".to_string());
//         return_handler.add_token(TokenEnvChange::new_database_change(
//             new_database.clone(),
//             new_database.to_string(),
//         ));
//         return_handler.add_token(TokenInfo::new(
//             &*client.tds_server_context(),
//             5701,
//             2,
//             0,
//             format!("Changed database context to '{}'", new_database.clone()),
//         ));
//         client.set_database(new_database);

//         // set collation change
//         // return_msg.add_token(TokenEnvChange::new_collation_change(
//         //     "".to_string(),
//         //     "".to_string(),
//         // ));

//         // set language change
//         return_handler.add_token(TokenEnvChange::new_language_change(
//             "".to_string(),
//             "us_english".to_string(),
//         ));
//         return_handler.add_token(TokenInfo::new(
//             &*client.tds_server_context(),
//             5703,
//             1,
//             0,
//             format!("Changed language to '{}'", "us_english"),
//         ));

//         // set packet size change
//         return_handler.add_token(TokenEnvChange::new_packet_size_change(
//             "4096".to_string(),
//             "4096".to_string(),
//         ));
//         return_handler.add_token(TokenInfo::new(
//             &*client.tds_server_context(),
//             5702,
//             1,
//             0,
//             format!("Changed packet size to {}", "4096"),
//         ));

//         // create login ack token
//         return_handler.add_token(TokenLoginAck::new(client.tds_server_context()));

//         // check if session recovery is enabled
//         if client.tds_server_context().session_recovery_enabled {
//             // msg.add_token(FeatureAck::new_session_recovery());
//         }

//         // create done token
//         return_handler.add_token(TokenDone::new_final());

//         // return_handler.flush().await?;
//         Ok(true)
//     }

//     fn on_federated_authentication_token_message(
//         &self,
//         session: &unilake_wire_frontend::prot::DefaultSession,
//     ) {
//         todo!()
//     }

//     async fn on_sql_batch_request<C>(&self, client: &mut C, msg: &BatchRequest) -> TdsWireResult<()>
//     where
//         C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
//     {
//         tracing::info!("Received SQL batch request: {}", msg.query);
//         // let mut response_handler = TdsBackendResponseHandler::new();
//         let mut response_handler = TdsBackendResponse::new(90, 9);
//         let mut column = TokenColMetaData::new();
//         let i = column.add_column("some_column", TypeInfo::FixedLen(FixedLenType::Int4));
//         column.column_set_updateable(i, true);
//         let column_len = column.len();
//         response_handler.add_token(column);

//         let mut count = 0;
//         for _ in 0..10 {
//             let mut row = TokenRow::new(column_len, false);
//             row.push(unilake_wire_frontend::tds::codec::ColumnData::I32(Some(1)));

//             response_handler.add_token(row);
//             count += 1;
//         }

//         response_handler.add_token(TokenDone::new_count(0, count));

//         client.feed(response_handler).await;
//         Ok(())
//     }

//     fn on_attention(&self, session: &unilake_wire_frontend::prot::DefaultSession) {
//         todo!()
//     }
// }
