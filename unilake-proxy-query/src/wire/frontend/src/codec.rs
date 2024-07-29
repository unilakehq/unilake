use std::io::Error as IOError;
use std::sync::Arc;

use derive_new::new;
use futures::future::poll_fn;
use futures::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::error::TdsWireError;
use crate::prot::{ServerInstance, SessionInfo, TdsWireHandlerFactory};
use crate::{TdsBackendResponse, TdsFrontendRequest, TdsMessageType, ALL_HEADERS_LEN_TX};

#[non_exhaustive]
#[derive(Debug, new)]
pub struct TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    pub session_info: S,
}

impl<S> Decoder for TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    type Item = TdsFrontendRequest;
    type Error = TdsWireError;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        // sanity checks on network level are done here, fully decoding is done afterwards
        if let Some(_header) = src.get(ALL_HEADERS_LEN_TX) {
            // todo(mrhamburg): check if header is valid in size (no overflows)
        } else {
            // wait for more data
            return Ok(None);
        }

        // todo(mrhamburg): do other checks (client ip, firewall, etc..)

        // do decoding
        TdsFrontendRequest::decode(src)
    }
}

impl<S> Encoder<TdsBackendResponse> for TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    type Error = TdsWireError;

    fn encode(&mut self, item: TdsBackendResponse, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode(dst).map_err(Into::into)
    }
}

impl<T, S> SessionInfo for Framed<T, TdsWireMessageServerCodec<S>>
where
    S: SessionInfo,
    T: Send + Sync,
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
        self.codec().session_info.sql_user_id()
    }

    fn database(&self) -> &str {
        self.codec().session_info.database()
    }

    fn tds_version(&self) -> &str {
        self.codec().session_info.tds_version()
    }

    fn connection_reset_request_count(&self) -> usize {
        self.codec().session_info.connection_reset_request_count()
    }

    fn tds_server_context(&self) -> Arc<crate::tds::server_context::ServerContext> {
        todo!()
    }

    fn set_server_nonce(&mut self, nonce: [u8; 32]) {
        self.codec_mut().session_info.set_server_nonce(nonce);
    }

    fn get_server_nonce(&self) -> Option<[u8; 32]> {
        self.codec().session_info.get_server_nonce()
    }

    fn set_client_nonce(&mut self, nonce: [u8; 32]) {
        self.codec_mut().session_info.set_client_nonce(nonce);
    }

    fn get_client_nonce(&self) -> Option<[u8; 32]> {
        self.codec().session_info.get_client_nonce()
    }

    fn increment_packet_id(&mut self) -> u8 {
        self.codec_mut().session_info.increment_packet_id()
    }

    fn get_packet_id(&self) -> u8 {
        self.codec().session_info.get_packet_id()
    }
}

pub type TdsWireResult<T> = Result<T, TdsWireError>;
async fn process_request<T, H, S>(
    request: TdsFrontendRequest,
    socket: &mut Framed<T, TdsWireMessageServerCodec<S>>,
    handlers: Arc<H>,
) -> TdsWireResult<()>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    S: SessionInfo,
    H: TdsWireHandlerFactory<S>,
{
    // todo(mrhamburg): implement state machine

    for (_header, message) in request.messages {
        match socket.state() {
            crate::prot::TdsSessionState::PreLoginSent => {
                if let TdsMessageType::PreLogin(p) = message {
                    handlers.on_prelogin_request(socket, &p).await?;
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
    }
    todo!()
}

pub async fn process_socket<H, S>(
    tcp_socket: TcpStream,
    tls_acceptor: Option<Arc<TlsAcceptor>>,
    handler: Arc<H>,
    instance: Arc<ServerInstance>,
) -> Result<(), IOError>
where
    S: SessionInfo,
    H: TdsWireHandlerFactory<S>,
{
    let addr = tcp_socket.peer_addr()?;
    tcp_socket.set_nodelay(true)?;

    let session_info = handler.open_session(&addr, &instance.clone());

    let session_info = match session_info {
        Ok(s) => {
            instance.increment_session_counter().await;
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
            if let Err(e) = process_request(msg, &mut socket, handler.clone()).await {
                todo!();
                // todo(mrhamburg): error handling + close session on error
                instance.decrement_session_counter().await;
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
