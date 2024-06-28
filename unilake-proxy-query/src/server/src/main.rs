use std::borrow::Borrow;
use std::env;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use unilake_wire_protocol::tds;

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

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            println!("New connection!");

            // In a loop, read data from the socket and write the data back.
            loop {
                let mut buf = [0; 4096];
                // while let Ok(data) = socket.try_read(&mut buf) {
                //     // residual data
                //     println!("len: {}", data);
                // }
                // // 12032 bytes
                // break;

                let p = tds::codec::PacketHeader::decode(&mut socket).await.unwrap();
                println!("{}", p.id);
                let dc = tds::codec::PreloginMessage::decode(&mut socket)
                    .await
                    .unwrap();
                println!("{}", dc.instance_name.unwrap_or("unknown".to_string()));

                // We don't have anything to write back yet...
                while let Ok(data) = socket.try_read(&mut buf) {
                    // residual data
                    println!("Got {} unparsed data!", data);
                }
                break;
            }
        });
    }
}
