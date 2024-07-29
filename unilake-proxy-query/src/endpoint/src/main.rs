use async_trait::async_trait;
use futures::{Sink, SinkExt};
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use unilake_wire_frontend::codec::{process_socket, TdsWireResult};
use unilake_wire_frontend::prot::{
    DefaultSession, ServerInstance, SessionInfo, TdsWireHandlerFactory,
};
use unilake_wire_frontend::tds::codec::{
    PreloginMessage, TdsBackendResponse, TokenPreLoginFedAuthRequiredOption,
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
    ) -> Result<
        unilake_wire_frontend::prot::DefaultSession,
        unilake_wire_frontend::codec::TdsWireError,
    > {
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
        C: SessionInfo + Sink<TdsBackendResponse>,
    {
        let encryption = ServerContext::encryption_response(
            client.tds_server_context().as_ref(),
            msg.encryption,
        );

        let mut token = PreloginMessage::new();
        token.version = client.tds_server_context().server_version as u32;
        token.encryption = Some(encryption);
        token.mars = false;
        if let Some(nonce) = msg.nonce {
            client.set_client_nonce(nonce);
        }

        if client.tds_server_context().fed_auth_options
            == TokenPreLoginFedAuthRequiredOption::FedAuthRequired
        {
            token.fed_auth_required = match msg.fed_auth_required {
                Some(a) => Some(a),
                None => None,
            };

            if msg.nonce.is_some() {
                token.nonce = Some(unilake_wire_frontend::utils::generate_random_nonce());
                client.set_server_nonce(token.nonce.unwrap());
            }
        }

        let result = TdsBackendResponse::new(client, client);
        client.send(result);
        Ok(())
    }

    fn on_login7_request(&self, session: &unilake_wire_frontend::prot::DefaultSession) {
        todo!()
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
