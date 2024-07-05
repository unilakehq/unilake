use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use unilake_wire_frontend::codec::process_socket;
use unilake_wire_frontend::prot::{DefaultSession, ServerInstance, TdsWireHandlerFactory};
use unilake_wire_frontend::tds::codec::{PreloginMessage, TdsFrontendMessage};

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
        let instance = ServerInstance::new();
        let rwlock = Arc::new(RwLock::new(instance));
        let bgworker = rwlock.write().await.start_instance(rwlock.clone());
        (rwlock, bgworker)
    };

    loop {
        let (socket, _) = listener.accept().await?;
        let factory_ref = factory.clone();
        let instance_ref = instance.clone();

        tokio::spawn(async move { process_socket(socket, None, factory_ref, instance_ref).await });
    }
}

struct DefaultTdsHandlerFactory {}

#[allow(unused_variables)]
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
        Ok(DefaultSession::new(socket_addr.clone()))
    }

    fn close_session(&self, session: &unilake_wire_frontend::prot::DefaultSession) {
        todo!()
    }

    fn on_prelogin_request(
        &self,
        session: &unilake_wire_frontend::prot::DefaultSession,
        msg: &PreloginMessage,
    ) {
        dbg!(msg.sub_build);
        todo!()
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
