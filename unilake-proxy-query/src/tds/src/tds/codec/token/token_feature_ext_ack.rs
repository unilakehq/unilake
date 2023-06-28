use crate::{Result, TokenType, FEA_EXT_FEDAUTH, FEA_EXT_TERMINATOR};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Feature Extension Acknowledgement token [2.2.7.11]
/// Introduced in TDS 7.4, FEATUREEXTACK is used to send an optional acknowledge
/// message to the client for features that are defined in FeatureExt. The token stream
/// is sent only along with the LOGINACK in a Login Response message.
#[derive(Debug)]
pub struct TokenFeatureExtAck {
    pub features: Vec<FeatureAck>,
}

#[derive(Debug)]
pub enum FedAuthAck {
    SecurityToken { nonce: Option<[u8; 32]> },
}

#[derive(Debug)]
pub enum FeatureAck {
    FedAuth(FedAuthAck),
    SessionRecovery,
    GenericOption(Vec<u8>),
    Utf8Support(bool),
}

impl TokenFeatureExtAck {
    pub(crate) async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let mut features = Vec::new();
        loop {
            let feature_id = src.read_u8().await?;

            if feature_id == FEA_EXT_TERMINATOR {
                break;
            } else if feature_id == FEA_EXT_FEDAUTH {
                let data_len = src.read_u32_le().await?;

                let nonce = if data_len == 32 {
                    let mut n = [0u8; 32];
                    src.read_exact(&mut n).await?;

                    Some(n)
                } else if data_len == 0 {
                    None
                } else {
                    panic!("invalid Feature_Ext_Ack token");
                };

                features.push(FeatureAck::FedAuth(FedAuthAck::SecurityToken { nonce }))
            } else {
                unimplemented!("unsupported feature {}", feature_id)
            }
        }

        Ok(TokenFeatureExtAck { features })
    }

    pub(crate) async fn encode<W>(&mut self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TokenType::FeatureExtAck as u8).await?;
        for item in self.features.iter() {
            match item {
                FeatureAck::FedAuth(s) => match s {
                    FedAuthAck::SecurityToken { nonce } => {
                        dest.write_u8(FEA_EXT_FEDAUTH).await?;
                        dest.write_u32_le(nonce.unwrap().len() as u32).await?;
                        dest.write_all(&nonce.unwrap()).await?;
                    }
                },
                _ => unimplemented!("unsupported feature {:?}", item),
            }
        }

        dest.write_u8(FEA_EXT_TERMINATOR).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

    use crate::{FeatureAck, FedAuthAck, TokenFeatureExtAck, TokenType};

    #[tokio::test]
    async fn encode_decode_token_feature_ext_ack() {
        let mut input = TokenFeatureExtAck {
            features: vec![FeatureAck::FedAuth(FedAuthAck::SecurityToken {
                nonce: Some([0u8; 32]),
            })],
        };

        // arrange
        let (inner, outer) = tokio::io::duplex(256);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input.encode(&mut writer).await.expect("should be ok");
        writer.flush().await.expect("should be ok");

        // decode
        let token_type = reader.read_u8().await.unwrap();
        let result = TokenFeatureExtAck::decode(&mut reader).await.unwrap();

        // assert
        assert_eq!(token_type, TokenType::FeatureExtAck as u8);
        assert_eq!(result.features.len(), input.features.len());
    }
}
