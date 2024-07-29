use std::cell::Cell;

use crate::{
    codec::TdsWireResult, prot::SessionInfo, tds::codec::header, PacketHeader, PacketStatus,
    TdsMessageType,
};
use tokio_util::bytes::{Buf, BytesMut};

// Complete Frontend Request
pub struct TdsFrontendRequest {
    pub messages: Vec<(PacketHeader, TdsMessageType)>,
}

impl TdsFrontendRequest {
    pub fn decode(buf: &mut BytesMut) -> TdsWireResult<Option<Self>> {
        // todo(mrhamburg): improve error handling here! (no unwrap)
        // todo(mrhamburg): add error handling here as well (without unwrap)
        let mut messages = Vec::new();
        while buf.has_remaining() {
            let header = header::PacketHeader::decode(buf).unwrap();
            messages.push((header, TdsMessageType::decode(buf, header.ty)?));
            if header.status == PacketStatus::EndOfMessage {
                break;
            }
        }
        Ok(Some(Self { messages }))
    }
}

pub struct TdsBackendResponse {
    pub header: Cell<Option<header::PacketHeader>>,
    pub messages: Vec<TdsMessageType>,
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

    pub fn add_message(mut self, message: TdsMessageType) -> Self {
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
        buf.unsplit(result);

        Ok(())
    }
}
