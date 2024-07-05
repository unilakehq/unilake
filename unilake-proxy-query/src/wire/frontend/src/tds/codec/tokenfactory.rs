use crate::{
    codec::TdsWireResult,
    tds::codec::{header, pre_login},
    LoginMessage, PreloginMessage, Result, TdsToken, TokenDone, TokenEnvChange, TokenError,
    TokenInfo, TokenOrder, TokenType,
};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio_util::bytes::{Buf, BytesMut};

use super::batch_request::BatchRequest;

struct TokenFactory {}

impl TokenFactory {
    pub(crate) async fn create<R>(mut src: &mut R) -> Result<Vec<TdsToken>>
    where
        R: AsyncRead + Unpin,
    {
        let mut buff = vec![];

        loop {
            todo!("Detect eol");
            let token_type = match TokenType::try_from(src.read_u8().await?) {
                Ok(t) => t,
                Err(_) => {
                    todo!("Do some error handling here");
                }
            };

            match token_type {
                TokenType::ReturnStatus => {}
                TokenType::ColMetaData => {}
                TokenType::Error => {
                    buff.push(TdsToken::Error(TokenError::decode(&mut src).await?));
                }
                TokenType::Info => {
                    buff.push(TdsToken::Info(TokenInfo::decode(&mut src).await?));
                }
                TokenType::Order => {
                    buff.push(TdsToken::Order(TokenOrder::decode(&mut src).await?));
                }
                TokenType::ColInfo => {}
                TokenType::LoginAck => {}
                TokenType::Row => {}
                TokenType::Sspi => {}
                TokenType::EnvChange => {
                    buff.push(TdsToken::EnvChange(TokenEnvChange::decode(&mut src).await?));
                }
                TokenType::Done => {
                    buff.push(TdsToken::Done(TokenDone::decode(&mut src).await?));
                }
                TokenType::DoneProc => {}
                TokenType::DoneInProc => {}
                TokenType::FeatureExtAck => {}
                TokenType::FedAuthInfo => {}
                TokenType::SessionState => {}
                _ => todo!("unexpected token type: {:?}", token_type),
            }
        }

        Ok(buff)
    }
}

/// Messages sent from Frontend
pub enum TdsFrontendMessage {
    PreLogin(PreloginMessage),
    BatchRequest(BatchRequest),
    Login(LoginMessage),
}

impl TdsFrontendMessage {
    pub fn encode(&self, buf: &mut BytesMut) -> TdsWireResult<()> {
        todo!()
    }

    pub fn decode(buf: &mut BytesMut) -> TdsWireResult<Option<Self>> {
        // todo(mrhamburg): improve error handling here! (no unwrap)
        let header = header::PacketHeader::decode(buf).unwrap();
        let _ = match header.ty {
            crate::PacketType::PreLogin => pre_login::PreloginMessage::decode(buf),
            _ => todo!("Not implemented"),
        };
        todo!()
    }
}

pub enum TdsBackendMessage {}
