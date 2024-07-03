use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use unilake_wire_frontend::codec::process_socket;
use unilake_wire_frontend::prot::{DefaultSession, SessionInfo, TdsWireHandlerFactory};

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

    loop {
        let (mut socket, _) = listener.accept().await?;
        let factory_ref = factory.clone();

        tokio::spawn(async move { process_socket(socket, None, factory_ref).await });
    }
}

struct DefaultTdsHandlerFactory {}
impl TdsWireHandlerFactory<unilake_wire_frontend::prot::DefaultSession>
    for DefaultTdsHandlerFactory
{
}
