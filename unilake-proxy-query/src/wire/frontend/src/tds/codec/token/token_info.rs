use crate::tds::codec::{decode, encode};
use crate::{Result, TdsToken, TdsTokenCodec, TdsTokenType};
use std::mem::size_of;
use tokio_util::bytes::{Buf, BufMut, BytesMut};

/// Info Token [2.2.7.13]
/// Used to send an information message to the client.
#[allow(dead_code)] // we might want to debug the values
#[derive(Debug)]
pub struct TokenInfo {
    /// info number
    pub number: u32,
    /// error state
    pub state: u8,
    /// severity (<10: Info)
    pub class: u8,
    pub message: String,
    pub server: String,
    pub procedure: String,
    pub line: u32,
}

impl TokenInfo {
    pub fn new(number: u32, state: u8, class: u8, message: String, server_name: String) -> Self {
        TokenInfo {
            number,
            state,
            class,
            message,
            server: server_name,
            procedure: String::new(),
            line: 0,
        }
    }
}

impl TdsTokenCodec for TokenInfo {
    fn decode(src: &mut BytesMut) -> Result<TdsToken> {
        let _length = src.get_u16_le();

        let number = src.get_u32_le();
        let state = src.get_u8();
        let class = src.get_u8();
        let message = decode::read_us_varchar(src)?;
        let server = decode::read_b_varchar(src)?;
        let procedure = decode::read_b_varchar(src)?;
        let line = src.get_u32_le();

        Ok(TdsToken::Info(TokenInfo {
            number,
            state,
            class,
            message,
            server,
            procedure,
            line,
        }))
    }

    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::Info as u8);

        let message_length = self.message.len();
        let server_length = self.server.len();
        let procedure_length = self.procedure.len();
        let length: u16 = (
            size_of::<u16>() // Number
                + (size_of::<u8>() * 2) // State + Class
                + size_of::<u16>() + message_length // Message
                + size_of::<u8>() + server_length // Server Name
                + size_of::<u8>() + procedure_length // Procedure Name
                + size_of::<u32>()
            // Line number
        ) as u16;

        dest.put_u16_le(length);
        dest.put_u32_le(self.number);
        dest.put_u8(self.state);
        dest.put_u8(self.class);
        encode::write_us_varchar(dest, &self.message);
        encode::write_b_varchar(dest, &self.server);
        encode::write_b_varchar(dest, &self.procedure);
        dest.put_u32_le(self.line);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::bytes::{Buf, BytesMut};

    use crate::{Result, TdsToken, TdsTokenCodec, TdsTokenType, TokenInfo};

    #[test]
    fn encode_decode_token_info() -> Result<()> {
        let input = TokenInfo {
            line: 12,
            class: 1,
            number: 1321,
            message: String::from("Hello World"),
            state: 2,
            procedure: String::from("sp.my_proc"),
            server: String::from("mydatabase"),
        };

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).expect("should be ok");

        // decode
        let tokentype = buff.get_u8();
        let result = TokenInfo::decode(&mut buff).unwrap();

        // assert
        assert_eq!(tokentype, TdsTokenType::Info as u8);
        if let TdsToken::Info(result) = result {
            assert_eq!(result.server, input.server);
            assert_eq!(result.message, input.message);
            assert_eq!(result.procedure, input.procedure);
            assert_eq!(result.class, input.class);
            assert_eq!(result.line, input.line);
            assert_eq!(result.state, input.state);
            assert_eq!(result.number, input.number);
        }

        Ok(())
    }
}
