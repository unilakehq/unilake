use crate::*;
use tokio_util::bytes::BytesMut;

pub enum TdsToken {
    Done(TokenDone),
    EnvChange(TokenEnvChange),
    Error(TokenError),
    Info(TokenInfo),
    Order(TokenOrder),
    FeatureExtAck(TokenFeatureExtAck),
    ColMetaData(TokenColMetaData),
    FedAuth(TokenFedAuth),
    LoginAck(TokenLoginAck),
    ReturnValue(TokenReturnValue),
    // todo(mrhamburg): see where tokenrow needs lifetime operators and best way forward
    // Row(TokenRow),
    Sspi(TokenSspi),
}

pub trait TdsTokenCodec {
    fn encode(&self, dst: &mut BytesMut) -> Result<()>;
    fn decode(src: &mut BytesMut) -> Result<TdsToken>;
}
