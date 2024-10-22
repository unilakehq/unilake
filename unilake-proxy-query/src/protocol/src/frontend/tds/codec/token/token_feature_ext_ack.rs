use crate::frontend::{
    utils::ReadAndAdvance, FeatureExt, Result, TdsToken, TdsTokenCodec, TdsTokenType,
};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

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
    SecurityToken { nonce: Option<Vec<u8>> },
}

#[derive(Debug)]
pub enum FeatureAck {
    FedAuth(FedAuthAck),
    SessionRecovery,
    GenericOption(Vec<u8>),
    Utf8Support(bool),
}

impl FeatureAck {
    pub fn new_session_recovery() -> Self {
        FeatureAck::SessionRecovery
    }
}

impl TdsTokenCodec for TokenFeatureExtAck {
    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::FeatureExtAck as u8);
        for item in self.features.iter() {
            match item {
                FeatureAck::FedAuth(s) => match s {
                    FedAuthAck::SecurityToken { nonce } => {
                        dest.put_u8(FeatureExt::FedAuth as u8);
                        let len = nonce.as_ref().unwrap().len();
                        dest.put_u32_le(len as u32);
                        dest.put_slice(nonce.as_ref().unwrap());
                    }
                },
                _ => unimplemented!("unsupported feature {:?}", item),
            }
        }

        dest.put_u8(FeatureExt::Terminator as u8);
        Ok(())
    }

    fn decode(src: &mut BytesMut) -> Result<TdsToken> {
        let mut features = Vec::new();
        loop {
            let feature_id = src.get_u8();

            if feature_id == FeatureExt::Terminator as u8 {
                break;
            } else if feature_id == FeatureExt::FedAuth as u8 {
                let data_len = src.get_u32_le();

                let nonce = if data_len == 32 {
                    let (_, n) = src.read_and_advance(32);
                    Some(n.to_vec())
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

        Ok(TdsToken::FeatureExtAck(TokenFeatureExtAck { features }))
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::bytes::{Buf, BytesMut};

    use crate::frontend::{
        FeatureAck, FedAuthAck, TdsToken, TdsTokenCodec, TdsTokenType, TokenFeatureExtAck,
    };

    #[test]
    fn encode_decode_token_feature_ext_ack() {
        let input = TokenFeatureExtAck {
            features: vec![FeatureAck::FedAuth(FedAuthAck::SecurityToken {
                nonce: Some(Vec::from([0u8; 32])),
            })],
        };

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).expect("should be ok");

        // decode
        let token_type = buff.get_u8();
        let result = TokenFeatureExtAck::decode(&mut buff).unwrap();

        // assert
        assert_eq!(token_type, TdsTokenType::FeatureExtAck as u8);
        if let TdsToken::FeatureExtAck(result) = result {
            assert_eq!(result.features.len(), input.features.len());
        }
    }
}
