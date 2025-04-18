use futures::stream::Abortable;
use std::net::SocketAddr;
use tokio_stream::wrappers::TcpListenerStream;
use unilake_common::error::Result;

pub type ListeningStream = Abortable<TcpListenerStream>;

#[async_trait::async_trait]
pub trait FrontendServer: Send {
    async fn start(&mut self, bind: SocketAddr) -> Result<SocketAddr>;
    async fn stop(&mut self, graceful: bool);
}
