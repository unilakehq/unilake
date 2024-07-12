use std::io::Error as IOError;
use std::sync::Arc;
use std::time::Duration;

use derive_new::new;
use futures::future::poll_fn;
use futures::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_rustls::TlsAcceptor;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::prot::{ServerInstance, SessionInfo, TdsWireHandlerFactory};
use crate::TdsFrontendMessage;

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

impl<S> Decoder for TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    type Item = TdsFrontendMessage;
    type Error = TdsWireError;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        match self.session_info.state() {
            _ => TdsFrontendMessage::decode(src),
        }
    }
}

impl<S> Encoder<TdsFrontendMessage> for TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    type Error = TdsWireError;

    fn encode(&mut self, item: TdsFrontendMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode(dst).map_err(Into::into)
    }
}

impl<T, S> SessionInfo for Framed<T, TdsWireMessageServerCodec<S>>
where
    S: SessionInfo,
{
    fn socket_addr(&self) -> std::net::SocketAddr {
        self.codec().session_info.socket_addr()
    }

    fn state(&self) -> &crate::prot::TdsSessionState {
        self.codec().session_info.state()
    }

    fn set_state(&mut self, new_state: crate::prot::TdsSessionState) {
        self.codec_mut().session_info.set_state(new_state)
    }

    fn session_id(&self) -> usize {
        self.codec().session_info.session_id()
    }

    fn packet_size(&self) -> usize {
        self.codec().session_info.packet_size()
    }

    fn sql_user_id(&self) -> &str {
        &self.codec().session_info.sql_user_id()
    }

    fn database(&self) -> &str {
        &self.codec().session_info.database()
    }

    fn tds_version(&self) -> &str {
        &self.codec().session_info.tds_version()
    }

    fn connection_reset_request_count(&self) -> usize {
        self.codec().session_info.connection_reset_request_count()
    }

    fn tds_server_context(&self) -> Arc<crate::tds::server_context::ServerContext> {
        todo!()
    }
}

pub type TdsWireResult<T> = Result<T, TdsWireError>;
async fn process_message<T, H, S>(
    message: TdsFrontendMessage,
    socket: &mut Framed<T, TdsWireMessageServerCodec<S>>,
    handlers: Arc<H>,
) -> TdsWireResult<()>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    S: SessionInfo,
    H: TdsWireHandlerFactory<S>,
{
    // todo(mrhamburg): implement state machine
    match socket.state() {
        crate::prot::TdsSessionState::PreLoginSent => {
            if let TdsFrontendMessage::PreLogin(p) = &message {
                handlers.on_prelogin_request(&socket.codec().session_info, p);
            }
            todo!()
        }
        crate::prot::TdsSessionState::SSLNegotiationSent => todo!(),
        crate::prot::TdsSessionState::CompleteLogin7Sent => todo!(),
        crate::prot::TdsSessionState::Login7SPNEGOSent => todo!(),
        crate::prot::TdsSessionState::Login7FederatedAuthenticationInformationRequestSent => {
            todo!()
        }
        crate::prot::TdsSessionState::LoggedIn => todo!(),
        crate::prot::TdsSessionState::RequestSent => todo!(),
        crate::prot::TdsSessionState::AttentionSent => todo!(),
        crate::prot::TdsSessionState::ReConnect => todo!(),
        crate::prot::TdsSessionState::LogoutSent => todo!(),
        crate::prot::TdsSessionState::Final => todo!(),
    }
    todo!()
}

pub async fn process_socket<H, S>(
    tcp_socket: TcpStream,
    tls_acceptor: Option<Arc<TlsAcceptor>>,
    handler: Arc<H>,
    instance: Arc<RwLock<ServerInstance>>,
) -> Result<(), IOError>
where
    S: SessionInfo,
    H: TdsWireHandlerFactory<S>,
{
    let addr = tcp_socket.peer_addr()?;
    tcp_socket.set_nodelay(true)?;

    let session_info = {
        let instance_info_ref = instance.read().await;
        handler.open_session(&addr, &instance_info_ref)
    };

    let session_info = match session_info {
        Ok(s) => {
            instance
                .read()
                .await
                .get_sender()
                .send(crate::prot::ServerInstanceMessage::IncrementSessionCounter)
                .unwrap();
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
        tokio::time::sleep(Duration::from_secs(12)).await;
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
        // socket.send(response).await?;
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
    return Ok(false);
    todo!();
    // if let Ok(Some(_)) = SslRequest::decode(&mut buf) {
    //     return Ok(true);
    // }
    Ok(false)
}
