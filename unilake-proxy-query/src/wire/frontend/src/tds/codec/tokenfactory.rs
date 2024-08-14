use super::{ResponseMessage, TdsToken};
use crate::{
    error::TdsWireResult, prot::SessionInfo, tds::codec::header, PacketHeader, TdsMessage,
    ALL_HEADERS_LEN_TX,
};
use std::cell::Cell;
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
    pub header: Cell<Option<header::PacketHeader>>,
    pub messages: Vec<TdsMessage>,
}

impl TdsBackendResponse {
    /// Create a new backend response
    pub fn new(session: &mut dyn SessionInfo) -> Self {
        TdsBackendResponse {
            header: Cell::new(Some(Self::get_next_header(session))),
            messages: Vec::new(),
        }
    }

    fn get_next_header(_: &mut dyn SessionInfo) -> PacketHeader {
        // PacketHeader::new(0, session.increment_packet_id())
        PacketHeader::new(0, 1)
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
    }

    /// Encode the response into a byte buffer
    pub fn encode(&self, buf: &mut BytesMut) -> TdsWireResult<()> {
        let header = self.header.replace(None);
        if header.is_none() {
            // return Err::new("No header available for encoding");
            todo!();
        }
        let mut header = header.unwrap();

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
