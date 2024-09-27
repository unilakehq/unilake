use std::io::Error as IOError;
use std::sync::Arc;

use derive_new::new;
use futures::future::poll_fn;
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tokio_util::bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder, Framed};
use ulid::Ulid;

use crate::frontend::error::{TdsWireError, TdsWireResult};
use crate::frontend::prot::{ServerInstance, SessionInfo, TdsSessionState, TdsWireHandlerFactory};
use crate::frontend::{
    PacketHeader, TdsBackendResponse, TdsFrontendRequest, TdsMessage, ALL_HEADERS_LEN_TX,
    MAX_PACKET_SIZE,
};

#[non_exhaustive]
#[derive(Debug)]
pub struct TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    pub session_info: S,
    packet_number: u8,
    current_response: BytesMut,
}

impl<S> TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    fn new(session_info: S) -> Self {
        TdsWireMessageServerCodec {
            session_info,
            packet_number: 0,
            current_response: BytesMut::new(),
        }
    }

    fn flush_response(&mut self, dst: &mut BytesMut, is_done: bool) -> Result<(), TdsWireError> {
        while self.current_response.has_remaining() {
            // get the length (or maximum length of the packet)
            let len = std::cmp::min(self.max_packet_size(), self.current_response.len());

            // get the slice for given size
            let slice = &self.current_response.split_to(len);

            // create header
            let mut header = self.get_next_header();
            header.length = (len + ALL_HEADERS_LEN_TX) as u16;
            header.is_end_of_message = header.length != self.session_info.packet_size();
            header.encode(dst)?;

            // get slice for given size
            dst.extend_from_slice(slice);

            if header.is_end_of_message && self.current_response.has_remaining() {
                panic!("Cannot send more data after sending EOM packet");
            }

            // wait for more data if we are not done yet
            if !is_done {
                break;
            }
        }

        // only clear if we are done
        if is_done {
            self.clear();
        }
        Ok(())
    }

    fn get_next_header(&mut self) -> PacketHeader {
        self.packet_number = self.packet_number.saturating_add(1);
        PacketHeader::new(0, self.packet_number)
    }

    fn clear(&mut self) {
        self.current_response = BytesMut::new();
        self.packet_number = 0;
    }

    fn max_packet_size(&self) -> usize {
        self.session_info.packet_size() as usize - ALL_HEADERS_LEN_TX
    }
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
        if let Some(header) = src.get(..ALL_HEADERS_LEN_TX) {
            // check if header is correct and as expected
            let mut buff = BytesMut::from(header);
            let header = PacketHeader::decode(&mut buff)?;
            // check header length compared to a defined maximum
            if header.length as usize > MAX_PACKET_SIZE || src.len() > MAX_PACKET_SIZE {
                return Err(TdsWireError::Protocol(
                    "Invalid packet size, too large".to_string(),
                ));
            }
        } else {
            // wait for more data
            return Ok(None);
        }

        // do decoding
        let result = TdsFrontendRequest::decode(src);
        if let Err(ref e) = result {
            tracing::error!("Error decoding message: {}", e);
        }

        // check if all data has been consumed
        // todo(mrhamburg), in case of residual bytes close the connection and check protocol if this is expected behaviour
        if !src.is_empty() {
            let msg = format!(
                "Incomplete packet received or processed ({} remaining), closing connection",
                src.len()
            );
            tracing::error!(msg);
        }
        result
    }
}

impl<S> Encoder<TdsBackendResponse> for TdsWireMessageServerCodec<S>
where
    S: SessionInfo,
{
    type Error = TdsWireError;

    fn encode(&mut self, item: TdsBackendResponse, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            TdsBackendResponse::Token(t) => {
                t.encode(&mut self.current_response)?;
            }
            TdsBackendResponse::Message(m) => {
                m.encode(&mut self.current_response)?;
            }
            TdsBackendResponse::Done => {
                // flush the response immediately upon receiving a Done message
                self.flush_response(dst, true)?;
                return Ok(());
            }
        }

        // flush when the current response exceeds the session packet size
        if self.current_response.len() > self.session_info.packet_size() as usize {
            self.flush_response(dst, false)?;
        }

        Ok(())
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

    fn state(&self) -> &TdsSessionState {
        self.codec().session_info.state()
    }

    fn set_state(&mut self, new_state: TdsSessionState) {
        self.codec_mut().session_info.set_state(new_state)
    }

    fn session_id(&self) -> Ulid {
        self.codec().session_info.session_id()
    }

    fn packet_size(&self) -> u16 {
        self.codec().session_info.packet_size()
    }

    fn get_sql_user_id(&self) -> &str {
        self.codec().session_info.get_sql_user_id()
    }

    fn get_database(&self) -> &str {
        self.codec().session_info.get_database()
    }

    fn tds_version(&self) -> &str {
        self.codec().session_info.tds_version()
    }

    fn connection_reset_request_count(&self) -> usize {
        self.codec().session_info.connection_reset_request_count()
    }

    fn tds_server_context(&self) -> Arc<crate::frontend::tds::server_context::ServerContext> {
        self.codec().session_info.tds_server_context().clone()
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

    fn set_sql_user_id(&mut self, sql_user_id: String) {
        self.codec_mut().session_info.set_sql_user_id(sql_user_id)
    }

    fn set_database(&mut self, db_name: String) {
        self.codec_mut().session_info.set_database(db_name)
    }

    fn get_catalog(&self) -> Option<&String> {
        self.codec().session_info.get_catalog()
    }

    fn set_catalog(&mut self, catalog: String) {
        self.codec_mut().session_info.set_catalog(catalog)
    }
}

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
    let incorrect_state_error = |expected: String| {
        TdsWireError::Protocol(format!(
            "Invalid session state, expected {expected} message"
        ))
    };

    for (_header, message) in request.messages {
        match socket.state() {
            TdsSessionState::Initial => {
                if let TdsMessage::PreLogin(p) = message {
                    handlers.on_prelogin_request(socket, &p).await?;
                    socket.set_state(TdsSessionState::PreLoginProcessed);
                } else {
                    return Err(incorrect_state_error("PreLogin".to_string()));
                }
            }
            TdsSessionState::PreLoginProcessed => {
                if let TdsMessage::Login(l) = message {
                    handlers.on_login7_request(socket, &l).await?;
                    socket.set_state(TdsSessionState::LoggedIn);
                } else {
                    return Err(incorrect_state_error("Login".to_string()));
                }
            }
            TdsSessionState::SSLNegotiationProcessed => todo!(),
            TdsSessionState::CompleteLogin7Processed => todo!(),
            TdsSessionState::Login7SPNEGOProcessed => todo!(),
            TdsSessionState::Login7FederatedAuthenticationInformationRequestProcessed => todo!(),
            TdsSessionState::LoggedIn => {
                if let TdsMessage::BatchRequest(b) = message {
                    handlers.on_sql_batch_request(socket, &b).await?;
                }
                // todo(mrhamburg): implement error handling for specific message types which we do not expect here
            }
            TdsSessionState::RequestReceived => todo!(),
            TdsSessionState::AttentionReceived => todo!(),
            TdsSessionState::ReConnect => todo!(),
            TdsSessionState::LogoutProcessed => todo!(),
            TdsSessionState::Final => todo!(),
        }
    }
    // todo(mrhamburg): improve this section
    handlers.flush(socket).await?;
    let result = socket.flush().await;
    if result.is_err() {
        panic!("Error flushing socket: {}", result.unwrap_err());
    }
    Ok(())
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

    let session_info = handler.open_session(&addr, instance.clone());

    let session_info = match session_info {
        Ok(s) => {
            instance.increment_session_counter();
            s
        }
        Err(_) => {
            // process_error(&mut socket, e).await?;
            return Ok(());
        }
    };

    let mut tcp_socket = Framed::new(tcp_socket, TdsWireMessageServerCodec::new(session_info));
    let ssl = peek_for_sslrequest(&mut tcp_socket, tls_acceptor.is_some()).await?;

    if !ssl {
        let mut socket = tcp_socket;

        while let Some(packet) = socket.next().await {
            match packet {
                Ok(msg) => {
                    if let Err(e) = process_request(msg, &mut socket, handler.clone()).await {
                        tracing::error!("Error processing request: {}", e);
                        // todo(mrhamburg): error handling + close session on error
                        // process_error(&mut socket, e).await?;
                        todo!()
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading packet: {}", e);
                    // todo(mrhamburg): error handling + close session on error
                    // session_info.close_session().await?;
                    socket.close().await?;
                }
            }
        }

        // remove session
        instance.decrement_session_counter();
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
    let ssl = false;
    if is_sslrequest_pending(socket.get_ref()).await? {
        // consume request
        socket.next().await;

        let _response = if ssl_supported {
            // ssl = true;
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

    let _buf = BytesMut::from(buf.filled());
    return Ok(false);
    todo!();
    // if let Ok(Some(_)) = SslRequest::decode(&mut buf) {
    //     return Ok(true);
    // }
    Ok(false)
}
