use super::{batch_request::BatchRequest, ResponseMessage};
use crate::frontend::tds::codec::rpc_request::RpcRequest;
use crate::frontend::{AttentionSignal, LoginMessage, PacketType, PreloginMessage, TokenFedAuth};
use tokio_util::bytes::BytesMut;
use unilake_common::error::TdsWireResult;

#[derive(Debug)]
pub enum TdsMessage {
    PreLogin(PreloginMessage),
    Login(LoginMessage),
    Response(ResponseMessage),
    BatchRequest(BatchRequest),
    FedAuth(TokenFedAuth),
    Attention(AttentionSignal),
    RemoteProcedureCall(RpcRequest),
}

impl TdsMessage {
    pub fn decode(buf: &mut BytesMut, packet_type: PacketType) -> TdsWireResult<TdsMessage> {
        // 2.2.1 Client Messages
        match packet_type {
            PacketType::PreLogin => PreloginMessage::decode(buf),
            PacketType::SQLBatch => BatchRequest::decode(buf),
            PacketType::PreTDSv7Login => PreloginMessage::decode(buf),
            PacketType::TDSv7Login => LoginMessage::decode(buf),
            PacketType::Rpc => RpcRequest::decode(buf),
            PacketType::Attention => AttentionSignal::decode(buf),

            // todo(mrhamburg): improve this, should return a specific error message
            PacketType::FederatedAuthenticationInfo => todo!(),
            PacketType::BulkLoad => todo!(),
            PacketType::TransactionManagerReq => todo!(),
            // _ => TdsWireError::new("Unknown message type").into(),
            _ => unimplemented!("unknown message type"),
        }
    }

    pub fn encode(&self, dst: &mut BytesMut) -> TdsWireResult<()> {
        // 2.2.2 Server Messages
        match self {
            TdsMessage::PreLogin(p) => p.encode(dst),
            TdsMessage::Login(l) => l.encode(dst),
            TdsMessage::Response(r) => r.encode(dst),

            // todo(mrhamburg): improve this, should return a specific error message
            // _ => TdsWireError::new("Unknown message type").into(),
            _ => unimplemented!("unknown message type"),
        }
    }
}

pub trait TdsMessageCodec {
    fn decode(src: &mut BytesMut) -> TdsWireResult<TdsMessage>
    where
        Self: Sized;
    fn encode(&self, dst: &mut BytesMut) -> TdsWireResult<()>;
}

macro_rules! impl_into_tdsmessage {
    ($t:ty, $tt:expr) => {
        impl Into<TdsMessage> for $t {
            fn into(self) -> TdsMessage {
                $tt(self)
            }
        }
    };
}

impl_into_tdsmessage!(PreloginMessage, TdsMessage::PreLogin);
impl_into_tdsmessage!(LoginMessage, TdsMessage::Login);
impl_into_tdsmessage!(ResponseMessage, TdsMessage::Response);
impl_into_tdsmessage!(BatchRequest, TdsMessage::BatchRequest);
impl_into_tdsmessage!(TokenFedAuth, TdsMessage::FedAuth);
impl_into_tdsmessage!(AttentionSignal, TdsMessage::Attention);
