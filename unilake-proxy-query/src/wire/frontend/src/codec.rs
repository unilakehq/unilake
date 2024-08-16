use std::io::Error as IOError;
use std::sync::Arc;

use derive_new::new;
use futures::future::poll_fn;
use futures::{Sink, SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::error::{TdsWireError, TdsWireResult};
use crate::prot::{ServerInstance, SessionInfo, TdsSessionState, TdsWireHandler};
use crate::{
    PacketHeader, TdsBackendResponse, TdsBackendResponseHandler, TdsFrontendRequest, TdsMessage,
    ALL_HEADERS_LEN_TX, MAX_PACKET_SIZE,
};

#[non_exhaustive]
#[derive(Debug, new)]
pub struct TdsWireMessageServerCodec {}

impl Decoder for TdsWireMessageServerCodec {
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

impl Encoder<TdsBackendResponse> for TdsWireMessageServerCodec {
    type Error = TdsWireError;

    fn encode(&mut self, item: TdsBackendResponse, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode(dst).map_err(Into::into)
    }
}

// async fn test_method<'a, T, H, S>(
//     request: TdsFrontendRequest,
//     session: &mut S,
//     handlers: Arc<H>,
//     thingy: TdsBackendResponseHandler<'a, Framed<TcpStream, TdsWireMessageServerCodec>>,
// ) -> TdsWireResult<()>
// where
//     T: Sink<TdsBackendResponse, Error = TdsWireError> + Unpin + Send,
//     S: SessionInfo,
//     H: TdsWireHandler<S, T>,
// {
//     handlers.on_prelogin_request(session, &mut thingy, request.messages.first());
//     todo!()
// }

async fn process_request<'a, H, S>(
    request: TdsFrontendRequest,
    response_handler: &mut TdsBackendResponseHandler<
        'a,
        Framed<TcpStream, TdsWireMessageServerCodec>,
    >,
    session: &mut S,
    handlers: Arc<H>,
) -> TdsWireResult<()>
where
    S: SessionInfo,
    H: TdsWireHandler<S>,
{
    // todo(mrhamburg): implement state machine
    let incorrect_state_error = |expected: String| {
        TdsWireError::Protocol(format!(
            "Invalid session state, expected {expected} message"
        ))
    };

    for (_header, message) in request.messages {
        let packet_size = session.packet_size();

        match session.state() {
            TdsSessionState::Initial => {
                if let TdsMessage::PreLogin(p) = message {
                    handlers
                        .on_prelogin_request(session, response_handler, &p)
                        .await?;
                    session.set_state(TdsSessionState::PreLoginProcessed);
                } else {
                    return Err(incorrect_state_error("PreLogin".to_string()));
                }
            }
            TdsSessionState::PreLoginProcessed => {
                if let TdsMessage::Login(l) = message {
                    if handlers
                        .on_login7_request(session, response_handler, &l)
                        .await?
                    {
                        session.set_state(TdsSessionState::LoggedIn);
                    }
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
                    handlers
                        .on_sql_batch_request(session, response_handler, &b)
                        .await?;
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
    Ok(())
}

pub async fn process_socket<S, H>(
    tcp_socket: TcpStream,
    tls_acceptor: Option<Arc<TlsAcceptor>>,
    handler: Arc<H>,
    instance: Arc<ServerInstance>,
) -> Result<(), IOError>
where
    S: SessionInfo,
    H: TdsWireHandler<S>,
{
    let addr = tcp_socket.peer_addr()?;
    tcp_socket.set_nodelay(true)?;

    let session_info = handler.open_session(&addr, instance.clone());

    let mut session_info = match session_info {
        Ok(s) => {
            instance.increment_session_counter();
            s
        }
        Err(_) => {
            // process_error(&mut socket, e).await?;
            return Ok(());
        }
    };

    let mut tcp_socket = Framed::new(tcp_socket, TdsWireMessageServerCodec::new());

    // let ssl = peek_for_sslrequest(&mut tcp_socket, tls_acceptor.is_some()).await?;
    let ssl = false;

    if !ssl {
        while let Some(packet) = tcp_socket.next().await {
            match packet {
                Ok(msg) => {
                    let mut response_handler = TdsBackendResponseHandler::new(&mut tcp_socket, 0);
                    // test_method(msg, &mut session_info, handler.clone(), response_handler);

                    if let Err(e) = process_request(
                        msg,
                        &mut response_handler,
                        &mut session_info,
                        handler.clone(),
                    )
                    .await
                    {
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
                    tcp_socket.close().await;
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
    socket: &mut Framed<TcpStream, TdsWireMessageServerCodec>,
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
