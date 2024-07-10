use crate::tds::codec::{decode, encode};
use crate::{Result, TdsTokenCodec};
use crate::{TdsToken, TdsTokenType};
use std::fmt;
use std::mem::size_of;
use tokio_util::bytes::{Buf, BufMut, BytesMut};

/// Error token [2.2.7.10]
/// Used to send an error message to the client.
#[derive(Clone, Debug)]
pub struct TokenError {
    /// ErrorCode
    pub code: u32,
    /// ErrorState (describing code)
    pub state: u8,
    /// The class (severity) of the error
    pub class: u8,
    /// The error message
    pub message: String,
    pub server: String,
    pub procedure: String,
    pub line: u32,
}

impl TokenError {
    pub fn new(
        code: u32,
        state: u8,
        class: u8,
        message: String,
        server: String,
        procedure: String,
        line: u32,
    ) -> TokenError {
        TokenError {
            code,
            state,
            class,
            message,
            server,
            procedure,
            line,
        }
    }
}

impl TdsTokenCodec for TokenError {
    fn decode(src: &mut BytesMut) -> Result<TdsToken> {
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

    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::Error as u8);

        let message_length = self.message.len();
        let server_length = self.server.len();
        let procedure_length = self.procedure.len();
        let length: u16 = (
            size_of::<u32>() // code
                + (size_of::<u8>() * 2) // state + class
                + size_of::<u16>() + message_length // message
                + size_of::<u8>() + server_length // server
                + size_of::<u8>() + procedure_length // procedure
                + size_of::<u32>()
            // Line number
        ) as u16;

        dest.put_u16_le(length);
        dest.put_u32_le(self.code);
        dest.put_u8(self.state);
        dest.put_u8(self.class);

        encode::write_us_varchar(dest, &self.message)?;
        encode::write_b_varchar(dest, &self.server)?;
        encode::write_b_varchar(dest, &self.procedure)?;

        dest.put_u32_le(self.line);

        Ok(())
    }
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' on server {} executing {} on line {} (code: {}, state: {}, class: {})",
            self.message, self.server, self.procedure, self.line, self.code, self.state, self.class
        )
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::bytes::{Buf, BytesMut};

    use crate::{TdsToken, TdsTokenCodec, TdsTokenType, TokenError};

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
