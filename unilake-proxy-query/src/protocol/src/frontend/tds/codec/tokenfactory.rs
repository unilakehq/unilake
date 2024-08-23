use super::TdsToken;
use crate::frontend::{error::TdsWireResult, tds::codec::header, PacketHeader, TdsMessage};
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
pub enum TdsBackendResponse {
    Token(TdsToken),
    Message(TdsMessage),
    Done,
}
