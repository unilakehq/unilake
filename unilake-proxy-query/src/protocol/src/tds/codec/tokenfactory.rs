use crate::{
    Result, TdsToken, TokenDone, TokenEnvChange, TokenError, TokenInfo, TokenOrder, TokenType,
};
use tokio::io::{AsyncRead, AsyncReadExt};

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
