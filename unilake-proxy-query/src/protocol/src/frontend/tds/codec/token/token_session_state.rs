use crate::frontend::{TdsToken, TdsTokenCodec, TdsTokenType};
use tokio_util::bytes::{BufMut, BytesMut};
use unilake_common::error::{TdsWireError, TdsWireResult};

/// SESSIONSTATE token [2.2.7.21]
/// For sending session state information to the client
#[derive(Debug)]
pub struct TokenSessionState {
    seq_no: u32,
    status: u8,
    state_id: u8,
    state_value: Vec<u8>,
}

impl TokenSessionState {
    pub fn new(seq_no: u32, status: u8, state_id: u8, state_value: Vec<u8>) -> TokenSessionState {
        TokenSessionState {
            seq_no,
            status,
            state_id,
            state_value,
        }
    }
}

impl TdsTokenCodec for TokenSessionState {
    fn encode(&self, dest: &mut BytesMut) -> TdsWireResult<()> {
        dest.put_u8(TdsTokenType::SessionState as u8);
        dest.put_u32_le(6 + self.state_value.len() as u32);
        dest.put_u32_le(self.seq_no);
        dest.put_u8(self.status);
        dest.put_u8(self.state_id);
        dest.put_u8(self.state_value.len() as u8);
        dest.put_slice(&self.state_value);
        Ok(())
    }

    fn decode(_: &mut BytesMut) -> TdsWireResult<TdsToken> {
        Err(TdsWireError::Protocol(
            "token(session_state): decode unsupported".into(),
        ))
    }
}
