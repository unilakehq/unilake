use std::io::Error as IOError;
use std::ops::Deref;
use std::sync::Arc;

use derive_new::new;
use futures::future::poll_fn;
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_rustls::TlsAcceptor;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::prot::{InstanceInfo, SessionInfo, TdsWireHandlerFactory};

#[non_exhaustive]
#[derive(Debug, new)]
pub struct TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    pub session_info: S,
}

// todo(mrhamburg): give these a nice spot
#[derive(thiserror::Error, Debug)]
pub enum TdsWireError {}
impl From<TdsWireError> for std::io::Error {
    fn from(e: TdsWireError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }
}
impl From<std::io::Error> for TdsWireError {
    fn from(value: std::io::Error) -> Self {
        todo!()
    }
}

pub struct TdsWireFrontendMessage {}
impl TdsWireFrontendMessage {
    pub fn decode(src: &mut BytesMut) -> Result<Option<Self>, TdsWireError> {
        todo!();
    }
}

pub struct TdsWireBackendMessage {}
impl TdsWireBackendMessage {
    pub fn encode(&self, dst: &mut BytesMut) -> Result<(), TdsWireError> {
        todo!()
    }
}

impl<S> Decoder for TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    type Item = TdsWireFrontendMessage;
    type Error = TdsWireError;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        match self.session_info.state() {
            _ => TdsWireFrontendMessage::decode(src),
        }
    }
}

impl<S> Encoder<TdsWireBackendMessage> for TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    type Error = TdsWireError;

    fn encode(
        &mut self,
        item: TdsWireBackendMessage,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        item.encode(dst).map_err(Into::into)
    }
}
pub type TdsWireResult<T> = Result<T, TdsWireError>;
async fn process_message<T, H, S>(
    message: TdsWireFrontendMessage,
    socket: &mut Framed<T, TdsWireMessageServerCodec<S>>,
    handlers: Arc<H>,
) -> TdsWireResult<()>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    S: SessionInfo,
    H: TdsWireHandlerFactory<S>,
{
    todo!()
}

pub async fn process_socket<H, S>(
    tcp_socket: TcpStream,
    tls_acceptor: Option<Arc<TlsAcceptor>>,
    handler: Arc<H>,
    instance_info: Arc<RwLock<InstanceInfo>>,
) -> Result<(), IOError>
where
    S: SessionInfo,
    H: TdsWireHandlerFactory<S>,
{
    let addr = tcp_socket.peer_addr()?;
    tcp_socket.set_nodelay(true)?;

    let session_info = {
        let instance_info_ref = instance_info.read().await;
        handler.open_session(&addr, &instance_info_ref)
    };

    let session_info = match session_info {
        Ok(s) => {
            instance_info.write().await.active_sessions += 1;
            s
        }
        Err(e) => {
            // process_error(&mut socket, e).await?;
            return Ok(());
        }
    };

    let mut tcp_socket = Framed::new(tcp_socket, TdsWireMessageServerCodec::new(session_info));
    let ssl = peek_for_sslrequest(&mut tcp_socket, tls_acceptor.is_some()).await?;

    if !ssl {
        let mut socket = tcp_socket;

        while let Some(Ok(msg)) = socket.next().await {
            if let Err(e) = process_message(msg, &mut socket, handler.clone()).await {
                todo!();
                // process_error(&mut socket, e).await?;
            }
        }
    }

    Ok(())
}

#[non_exhaustive]
#[derive(PartialEq, Eq, Debug, new)]
pub struct SslRequest;

impl SslRequest {
    pub const BODY_MAGIC_NUMBER: i32 = -1;
    pub const BODY_SIZE: usize = 8;
}

async fn peek_for_sslrequest<S>(
    socket: &mut Framed<TcpStream, TdsWireMessageServerCodec<S>>,
    ssl_supported: bool,
) -> Result<bool, IOError>
where
    S: SessionInfo,
{
    let mut ssl = false;
    if is_sslrequest_pending(socket.get_ref()).await? {
        // consume request
        socket.next().await;

        let response = if ssl_supported {
            ssl = true;
            // TdsWireBackendMessage::SslResponse(SslResponse::Accept)
            todo!()
        } else {
            // TdsWireBackendMessage::SslResponse(SslResponse::Refuse)
            todo!()
        };
        socket.send(response).await?;
    }
    Ok(ssl)
}

async fn is_sslrequest_pending(tcp_socket: &TcpStream) -> Result<bool, IOError> {
    let mut buf = [0u8; SslRequest::BODY_SIZE];
    let mut buf = ReadBuf::new(&mut buf);
    while buf.filled().len() < SslRequest::BODY_SIZE {
        if poll_fn(|cx| tcp_socket.poll_peek(cx, &mut buf)).await? == 0 {
            // the tcp_stream has ended
            return Ok(false);
        }
    }

    let mut buf = BytesMut::from(buf.filled());
    todo!();
    // if let Ok(Some(_)) = SslRequest::decode(&mut buf) {
    //     return Ok(true);
    // }
    Ok(false)
}
