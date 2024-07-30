use std::cell::Cell;

use crate::{
    error::TdsWireResult, prot::SessionInfo, tds::codec::header, PacketHeader, PacketStatus,
    TdsMessage, ALL_HEADERS_LEN_TX,
};
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

            // ignore packets with status "IgnoreEvent"
            if header.status != PacketStatus::IgnoreEvent {
                messages.push((header, TdsMessage::decode(buf, header.ty)?));
            }

            // no need to proceed with EOM messages
            if header.status == PacketStatus::EndOfMessage {
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
    pub fn new(session: &mut dyn SessionInfo) -> Self {
        let packet_header = PacketHeader::new(0, session.increment_packet_id());
        TdsBackendResponse {
            header: Cell::new(Some(packet_header)),
            messages: Vec::new(),
        }
    }

    pub fn add_message(mut self, message: TdsMessage) -> Self {
        self.messages.push(message);
        self
    }

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
