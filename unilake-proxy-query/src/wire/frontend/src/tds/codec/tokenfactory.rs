use super::{ResponseMessage, TdsToken};
use crate::{
    error::{TdsWireError, TdsWireResult},
    prot::SessionInfo,
    tds::codec::header,
    PacketHeader, TdsMessage, ALL_HEADERS_LEN_TX,
};
use futures::{Sink, SinkExt};
use std::cell::Cell;
use tokio::io::AsyncWrite;
use tokio_util::bytes::{Buf, BytesMut};

// Complete Frontend Request
pub struct TdsFrontendRequest {
    pub messages: Vec<(PacketHeader, TdsMessage)>,
}

impl TdsFrontendRequest {
    pub fn decode(buf: &mut BytesMut) -> TdsWireResult<Option<Self>> {
        let mut messages = Vec::new();
        while buf.has_remaining() {
            let header = header::PacketHeader::decode(buf)?;
            tracing::debug!(
                message = "Receiving packet",
                message_type = header.ty.to_string(),
                message_length = header.length
            );

            // ignore packets with status "IgnoreEvent" (0x01 must also be set)
            if header.is_ignore_event && header.is_end_of_message {
                continue;
            }

            // add message to result
            messages.push((header, TdsMessage::decode(buf, header.ty)?));

            // no need to proceed with EOM messages
            if header.is_end_of_message {
                break;
            }
        }
        Ok(Some(Self { messages }))
    }
}

#[derive(Debug)]
pub struct TdsBackendResponse {
    pub messages: Vec<TdsMessage>,
    max_packet_size: usize,
    packet_number: u8,
}

// todo: improve implementation, packet number is for a continuous sequence, so if you need multiple responses for a request
// todo: make sure we can send multiple responses for a single request (currently we can only send one), in case the len of the response exceeds the max packet size
impl TdsBackendResponse {
    /// Create a new backend response
    pub fn new(max_packet_size: u32, packet_number: u8) -> Self {
        TdsBackendResponse {
            messages: Vec::new(),
            max_packet_size: max_packet_size as usize,
            packet_number,
        }
    }

    fn len(&self) -> usize {
        todo!()
    }

    /// Add new message to the response
    pub fn add_message<T>(&mut self, message: T)
    where
        T: Into<TdsMessage>,
    {
        self.messages.push(message.into());
    }

    /// Add new token to the latest response message, if no response message exists a new one is created
    pub fn add_token<T>(&mut self, token: T)
    where
        T: Into<TdsToken>,
    {
        if let Some(TdsMessage::Response(r)) = self.messages.last_mut() {
            r.add_token(token.into());
        } else {
            let mut r = ResponseMessage::new();
            r.add_token(token.into());
            self.add_message(r);
        }

        if self.len() > self.max_packet_size {}
    }

    /// Encode the response into a byte buffer
    pub fn encode(&self, buf: &mut BytesMut) -> TdsWireResult<()> {
        let mut header = PacketHeader::result(self.packet_number);

        // encode all messages
        let mut result = BytesMut::new();
        for msg in self.messages.iter() {
            msg.encode(&mut result)?;
        }

        header.length = result.len() as u16;
        header.length += ALL_HEADERS_LEN_TX as u16;
        header.encode(buf)?;
        buf.unsplit(result);

        Ok(())
    }
}

pub struct TdsBackendResponseHandler<'a, T>
where
    T: Sink<TdsBackendResponse, Error = TdsWireError> + Sized + Unpin,
{
    pub response: Option<TdsBackendResponse>,
    pub sink: &'a mut T,
    max_packet_size: u32,
    packet_number: u8,
}

impl<'a, T> TdsBackendResponseHandler<'a, T>
where
    T: Sink<TdsBackendResponse, Error = TdsWireError> + Sized + Unpin,
{
    pub fn new(sink: &'a mut T, max_packet_size: u32) -> Self {
        TdsBackendResponseHandler {
            response: None,
            sink,
            max_packet_size,
            packet_number: 0,
        }
    }
    fn next_response(&mut self) {
        self.packet_number += 1;
        self.response = Some(TdsBackendResponse::new(
            self.max_packet_size,
            self.packet_number,
        ))
    }

    pub async fn flush(&mut self) -> TdsWireResult<()> {
        if let Some(r) = self.response.take() {
            self.sink.feed(r).await;
            self.sink.flush().await;
            todo!()
        }
        self.next_response();
        Ok(())
    }

    /// Add new message to the response
    pub async fn add_message<M>(&mut self, message: M)
    where
        M: Into<TdsMessage>,
    {
        if self.response.is_none() {
            self.next_response();
        }
        if let Some(r) = self.response.as_mut() {
            r.add_message(message)
        }
    }

    /// Add new token to the latest response message, if no response message exists a new one is created
    pub async fn add_token<M>(&mut self, token: M)
    where
        M: Into<TdsToken>,
    {
        if self.response.is_none() {
            self.next_response();
        }
        if let Some(r) = self.response.as_mut() {
            r.add_token(token)
        }
    }
}
