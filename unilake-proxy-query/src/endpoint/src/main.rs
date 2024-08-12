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
use unilake_wire_frontend::prot::{
    DefaultSession, ServerInstance, SessionInfo, TdsWireHandlerFactory,
};
use unilake_wire_frontend::tds::codec::{
    DoneStatus, EnvChangeType, FeatureAck, LoginMessage, OptionFlag2, PacketType, PreloginMessage,
    TdsBackendResponse, TdsMessage, TdsToken, TokenDone, TokenEnvChange, TokenInfo, TokenLoginAck,
    TokenPreLoginFedAuthRequiredOption,
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

    let factory = Arc::new(DefaultTdsHandlerFactory {});
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

struct DefaultTdsHandlerFactory {}

#[allow(unused_variables)]
#[async_trait]
impl TdsWireHandlerFactory<unilake_wire_frontend::prot::DefaultSession>
    for DefaultTdsHandlerFactory
{
    fn open_session(
        &self,
        socket_addr: &std::net::SocketAddr,
        instance_info: &ServerInstance,
    ) -> Result<unilake_wire_frontend::prot::DefaultSession, TdsWireError> {
        tracing::info!("New session for: {}", socket_addr);
        Ok(DefaultSession::new(
            socket_addr.clone(),
            instance_info.ctx.clone(),
        ))
    }

    fn close_session(&self, session: &unilake_wire_frontend::prot::DefaultSession) {
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
        prelogin_msg.version = server_context.server_version as u32;
        prelogin_msg.encryption = Some(encryption);
        prelogin_msg.mars = false;
        if let Some(nonce) = msg.nonce {
            client.set_client_nonce(nonce);
        }

        if server_context.fed_auth_options == TokenPreLoginFedAuthRequiredOption::FedAuthRequired {
            prelogin_msg.fed_auth_required = match msg.fed_auth_required {
                Some(a) => Some(a),
                None => None,
            };

            if msg.nonce.is_some() {
                prelogin_msg.nonce = Some(unilake_wire_frontend::utils::generate_random_nonce());
                client.set_server_nonce(prelogin_msg.nonce.unwrap());
            }
        }
        let mut msg = TdsBackendResponse::new(client);
        msg.add_message(prelogin_msg);
        client.feed(msg).await;

        Ok(())
    }

    async fn on_login7_request<C>(&self, client: &mut C, msg: &LoginMessage) -> TdsWireResult<()>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
    {
        if let Some(ref dbname) = msg.db_name {
            tracing::info!("Login request for database: {}", dbname);
        }

        // let database = msg.db_name.unwrap_or("main".to_string());
        let database = "".to_string();
        client.set_database(database);

        // todo(mrhamburg): check for tds version

        // check for fed auth
        if let Some(ref fed_auth_ext) = msg.fed_auth_ext {
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

        // Create return message
        let mut msg = TdsBackendResponse::new(client);

        // set database change
        msg.add_token(TokenEnvChange::new_database_change(
            "".to_string(),
            "".to_string(),
        ));
        // msg.add_token(TokenInfo::new(
        //     5701,
        //     2,
        //     0,
        //     format!("Changed database context to '{}'", database),
        //     client.tds_server_context().server_name.clone(),
        // ));

        // set collation change
        msg.add_token(TokenEnvChange::new_collation_change("".to_string()));

        // set language change
        // msg.add_token(TokenEnvChange::new_language_change("".to_string()));
        msg.add_token(TokenInfo::new(
            5703,
            1,
            0,
            format!("Changed language to '{}'", ""),
            "".to_string(),
        ));

        // set packet size change
        msg.add_token(TokenEnvChange::new_packet_size_change(
            "".to_string(),
            "".to_string(),
        ));
        msg.add_token(TokenInfo::new(
            5702,
            1,
            0,
            format!("Changed packet size to {}", ""),
            "".to_string(),
        ));

        // create login ack token
        // msg.add_token(TokenLoginAck::new(client.tds_server_context().server_name));

        // check if session recovery is enabled
        if client.tds_server_context().session_recovery_enabled {
            // msg.add_token(FeatureAck::new_session_recovery());
        }

        // create done token
        msg.add_token(TokenDone::new_final());

        client.feed(msg).await;
        Ok(())
    }

    fn on_federated_authentication_token_message(
        &self,
        session: &unilake_wire_frontend::prot::DefaultSession,
    ) {
        todo!()
    }

    fn on_sql_batch_request(&self, session: &unilake_wire_frontend::prot::DefaultSession) {
        todo!()
    }

    fn on_attention(&self, session: &unilake_wire_frontend::prot::DefaultSession) {
        todo!()
    }
}
