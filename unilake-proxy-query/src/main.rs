use tokio::io::{AsyncBufReadExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::{io, io::AsyncWriteExt, net::TcpListener, spawn};

#[tokio::main]
async fn main() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { panic!() });

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut listener = TcpListener::bind("127.0.0.1:8080").await?;

        loop {
            let (socket, _addr) = listener.accept().await?;
            // TODO: this will also prob. need to be cancellable
            spawn(async move {
                println!(
                    "Connection established with address {:?} and port {:?}",
                    reader.peer_addr(),
                    _addr.port()
                );
                handle_connection(socket)
            });
        }
    });
}

async fn handle_connection(socket: TcpStream) -> io::Result<()> {
    let (recv, send) = io::split(socket);
    let mut reader = BufReader::new(recv);
    let mut writer = BufWriter::new(send);

    // TODO: something like a session handler with connection state
    loop {
        let bytes_read = reader.read_line(&mut line).await;
        match bytes_read {
            Ok(_) => {
                if line.starts_with("exit") {
                    break;
                }
                writer.write_all(line.as_bytes()).await.unwrap();
                line.clear();
            }
            Err(_) => {
                break;
            }
        }
    }

    Ok(())
}
