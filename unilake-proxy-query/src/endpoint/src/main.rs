// this will probably also need a rest endpoint for front-end connections

use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use unilake_protocol::backend::starrocks::StarRocksTdsHandlerFactory;
use unilake_protocol::frontend::codec::process_socket;
use unilake_protocol::frontend::prot::ServerInstance;
use unilake_protocol::frontend::tds::server_context::ServerContext;

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
        .unwrap_or_else(|| "0.0.0.0:1433".to_string());

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    // todo(mrhamburg): use bgworker for graceful shutdown
    let (instance, _) = {
        let instance = ServerInstance::new(ServerContext::default());
        let (instance, bgworker) = instance.start_instance();
        (instance, bgworker)
    };

    let factory = Arc::new(StarRocksTdsHandlerFactory::new(instance.clone()));

    loop {
        let (socket, _) = listener.accept().await?;
        let factory = factory.clone();
        let instance = instance.clone();

        tokio::spawn(async move { process_socket(socket, None, factory, instance).await });
    }
}
