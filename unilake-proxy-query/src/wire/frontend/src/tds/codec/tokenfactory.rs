use std::cell::Cell;

use crate::{
    codec::TdsWireResult, prot::SessionInfo, tds::codec::header, PacketHeader, PacketStatus,
    TdsMessage, TdsToken,
};
use tokio_util::bytes::{Buf, BytesMut};

// Complete Frontend Request
pub struct TdsFrontendRequest {
    pub messages: Vec<(PacketHeader, TdsMessage)>,
}

impl TdsFrontendRequest {
    pub fn decode(buf: &mut BytesMut) -> TdsWireResult<Option<Self>> {
        // todo(mrhamburg): improve error handling here! (no unwrap)
        // todo(mrhamburg): add error handling here as well (without unwrap)
        let mut messages = Vec::new();
        while buf.has_remaining() {
            let header = header::PacketHeader::decode(buf)?;
            tracing::debug!(
                message = "Receiving packet",
                message_type = header.ty.to_string(),
                message_length = header.length
            );
            messages.push((header, TdsMessage::decode(buf, header.ty)?));
            if header.status == PacketStatus::EndOfMessage {
                break;
            }
        }
        Ok(Some(Self { messages }))
    }
}

pub struct TdsBackendResponse {
    pub header: Cell<Option<header::PacketHeader>>,
    pub messages: Vec<TdsMessage>,
}

impl TdsBackendResponse {
    pub fn new(mut packet_header: PacketHeader, session: &mut dyn SessionInfo) -> Self {
        // todo(mrhamburg): implement logic, include incrementing packet_id and setting session_id
        packet_header.id = session.increment_packet_id();
        TdsBackendResponse {
            header: Cell::new(Some(packet_header)),
            messages: Vec::new(),
        }
    }

    pub fn add_message(mut self, message: TdsMessage) -> Self {
        self.messages.push(message);
        self
    }

    pub fn add_token(mut self, token: TdsToken) -> Self {
        todo!()
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
        buf.unsplit(result);

        Ok(())
    }
}
