use crate::frontend::tds::codec::{decode, encode};
use crate::frontend::TdsTokenCodec;
use crate::frontend::{TdsToken, TdsTokenType};
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use unilake_common::error::{TdsWireResult, TokenError};

impl TdsTokenCodec for TokenError {
    fn encode(&self, dest: &mut BytesMut) -> TdsWireResult<()> {
        dest.put_u8(TdsTokenType::Error as u8);
        let mut buff = BytesMut::new();

        // set content
        buff.put_u32_le(self.code);
        buff.put_u8(self.state);
        buff.put_u8(self.class);

        encode::write_us_varchar(&mut buff, &self.message)?;
        encode::write_b_varchar(&mut buff, &self.server)?;
        encode::write_b_varchar(&mut buff, &self.procedure)?;

        buff.put_u32_le(self.line);

        // set length and push data
        dest.put_u16_le(buff.len() as u16);
        dest.extend_from_slice(&buff);

        Ok(())
    }

    fn decode(src: &mut BytesMut) -> TdsWireResult<TdsToken> {
        let _length = src.get_u16_le() as usize;

        let code = src.get_u32_le();
        let state = src.get_u8();
        let class = src.get_u8();

        let message = decode::read_us_varchar(src)?;
        let server = decode::read_b_varchar(src)?;
        let procedure = decode::read_b_varchar(src)?;

        let line = src.get_u32_le();

        let token = TokenError {
            code,
            state,
            class,
            message,
            server,
            procedure,
            line,
        };

        Ok(TdsToken::Error(token))
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::{TdsToken, TdsTokenCodec, TdsTokenType};
    use tokio_util::bytes::{Buf, BytesMut};
    use unilake_common::error::TokenError;

    #[test]
    fn encode_decode_token_error() {
        let input = TokenError {
            code: 12,
            server: String::from("mydatabase.some.domain.com"),
            message: String::from("There is something wrong here"),
            class: 2,
            line: 211,
            state: 1,
            procedure: String::from("my_procedure"),
        };

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).expect("should be ok");

        // decode
        let tokentype = buff.get_u8();
        let result = TokenError::decode(&mut buff).unwrap();

        // assert
        if let TdsToken::Error(result) = result {
            assert_eq!(tokentype, TdsTokenType::Error as u8);
            assert_eq!(result.code, input.code);
            assert_eq!(result.server, input.server);
            assert_eq!(result.message, input.message);
            assert_eq!(result.procedure, input.procedure);
            assert_eq!(result.class, input.class);
            assert_eq!(result.line, input.line);
            assert_eq!(result.state, input.state);
        } else {
            panic!("Expected TokenError")
        }
    }
}
