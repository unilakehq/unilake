use crate::frontend::{utils::ReadAndAdvance, TdsToken, TdsTokenType};
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use unilake_common::error::TdsWireResult;

/// Sspi token [2.2.7.22]
/// The SSPI token returned during the login process.
#[derive(Debug)]
pub struct TokenSspi(Vec<u8>);

impl TokenSspi {
    pub fn decode(src: &mut BytesMut) -> TdsWireResult<TdsToken> {
        let len = src.get_u16_le() as usize;
        let (_, bytes) = src.read_and_advance(len);

        Ok(TdsToken::Sspi(Self(bytes.to_vec())))
    }

    pub fn encode(&self, dest: &mut BytesMut) -> TdsWireResult<()> {
        dest.put_u8(TdsTokenType::Sspi as u8);
        dest.put_u16_le(self.0.len() as u16);
        dest.put_slice(&self.0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::{TdsToken, TdsTokenType, TokenSspi};
    use tokio_util::bytes::{Buf, BytesMut};
    use unilake_common::error::TdsWireResult;

    #[test]
    fn encode_decode_token_sspi() -> TdsWireResult<()> {
        let input = TokenSspi {
            0: vec![1, 2, 3, 4, 5, 6, 7],
        };

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff)?;

        // decode
        let tokentype = buff.get_u8();
        let result = TokenSspi::decode(&mut buff).unwrap();

        // assert
        assert_eq!(tokentype, TdsTokenType::Sspi as u8);
        if let TdsToken::Sspi(result) = result {
            assert_eq!(input.0, result.0);
        }

        Ok(())
    }
}
