use tokio_util::bytes::BytesMut;

use crate::{
    codec::TdsWireResult, LoginMessage, PacketType, PreloginMessage, TdsTokenCodec, TdsTokenType,
    TokenFedAuth,
};

use super::batch_request::BatchRequest;

#[derive(Debug)]
pub enum TdsMessageType {
    PreLogin(PreloginMessage),
    Login(LoginMessage),
    Response(Option<Vec<TdsTokenType>>),
    BatchRequest(BatchRequest),
    FedAuth(TokenFedAuth),
    Attention,
    RemoteProcedureCall,
}

impl TdsMessageType {
    pub fn decode(buf: &mut BytesMut, packet_type: PacketType) -> TdsWireResult<TdsMessageType> {
        // 2.2.1 Client Messages
        match packet_type {
            PacketType::PreLogin => PreloginMessage::decode(buf),
            PacketType::SQLBatch => BatchRequest::decode(buf),
            PacketType::PreTDSv7Login => todo!(),
            PacketType::TDSv7Login => LoginMessage::decode(buf),
            PacketType::FederatedAuthenticationInfo => todo!(),
            PacketType::BulkLoad => todo!(),
            PacketType::Rpc => todo!(),
            PacketType::AttentionSignal => todo!(),
            PacketType::TransactionManagerReq => todo!(),
            // _ => TdsWireError::new("Unknown message type").into(),
            _ => unimplemented!("unknown message type"),
        }
    }

    pub fn encode(&self, dst: &mut BytesMut) -> TdsWireResult<()> {
        // 2.2.2 Server Messages
        match self {
            TdsMessageType::PreLogin(p) => p.encode(dst)?,
            TdsMessageType::Login(l) => l.encode(dst)?,
            TdsMessageType::Response(r) => todo!(),
            TdsMessageType::BatchRequest(b) => b.encode(dst)?,
            // _ => TdsWireError::new("Unknown message type").into(),
            _ => unimplemented!("unknown message type"),
        }
        todo!()
    }
}

pub trait TdsMessage
where
    Self: Sized,
{
    fn decode(src: &mut BytesMut) -> TdsWireResult<TdsMessageType>
    where
        Self: Sized;
    fn encode(&self, dst: &mut BytesMut) -> TdsWireResult<()>;
}
