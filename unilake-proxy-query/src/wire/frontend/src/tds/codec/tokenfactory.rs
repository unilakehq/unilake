use crate::{
    codec::TdsWireResult,
    tds::codec::{header, pre_login},
    LoginMessage, PreloginMessage, Result, TdsToken, TdsTokenType, TokenDone, TokenEnvChange,
    TokenError, TokenInfo, TokenOrder,
};
use tokio_util::bytes::{Buf, BytesMut};

use super::batch_request::BatchRequest;

/// Messages sent from Frontend
pub enum TdsFrontendMessage {
    PreLogin(PreloginMessage),
    BatchRequest(BatchRequest),
    Login(LoginMessage),
    Response(Option<Vec<TdsTokenType>>),
}

impl TdsFrontendMessage {
    pub fn encode(&self, buf: &mut BytesMut) -> TdsWireResult<()> {
        todo!()
    }

    pub fn decode(buf: &mut BytesMut) -> TdsWireResult<Option<Self>> {
        // todo(mrhamburg): improve error handling here! (no unwrap)
        // todo(mrhamburg): check if we need to create a collection instead (not sure why though)
        // todo(mrhamburg): add error handling here as well (without unwrap)
        let header = header::PacketHeader::decode(buf).unwrap();

        match header.ty {
            crate::PacketType::PreLogin => TdsWireResult::Ok(Some(
                pre_login::PreloginMessage::decode(buf)
                    .map(|v| Self::PreLogin(v))
                    .unwrap(),
            )),
            _ => todo!("Not implemented"),
        }
    }
}

pub enum TdsBackendMessage {}
