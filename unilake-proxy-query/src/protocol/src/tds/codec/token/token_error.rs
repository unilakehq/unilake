use crate::tds::codec::{decode, encode};
use crate::Result;
use crate::TokenType;
use std::fmt;
use std::mem::size_of;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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

    pub async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let _length = src.read_u16_le().await? as usize;

        let code = src.read_u32_le().await?;
        let state = src.read_u8().await?;
        let class = src.read_u8().await?;

        let message = decode::read_us_varchar(src).await?;
        let server = decode::read_b_varchar(src).await?;
        let procedure = decode::read_b_varchar(src).await?;

        let line = src.read_u32_le().await?;

        let token = TokenError {
            code,
            state,
            class,
            message,
            server,
            procedure,
            line,
        };

        Ok(token)
    }

    pub async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TokenType::Error as u8).await?;

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

        dest.write_u16_le(length).await?;
        dest.write_u32_le(self.code).await?;
        dest.write_u8(self.state).await?;
        dest.write_u8(self.class).await?;

        encode::write_us_varchar(dest, &self.message).await?;
        encode::write_b_varchar(dest, &self.server).await?;
        encode::write_b_varchar(dest, &self.procedure).await?;

        dest.write_u32_le(self.line).await?;

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
    use crate::{TokenError, TokenType};
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

    #[tokio::test]
    async fn encode_decode_token_error() {
        let mut input = TokenError {
            code: 12,
            server: String::from("mydatabase.some.domain.com"),
            message: String::from("There is something wrong here"),
            class: 2,
            line: 211,
            state: 1,
            procedure: String::from("my_procedure"),
        };

        // arrange
        let (inner, outer) = tokio::io::duplex(256);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input.encode(&mut writer).await.expect("should be ok");
        writer.flush().await.expect("should be ok");

        // decode
        let tokentype = reader.read_u8().await.unwrap();
        let result = TokenError::decode(&mut reader).await.unwrap();

        // assert
        assert_eq!(tokentype, TokenType::Error as u8);
        assert_eq!(result.code, input.code);
        assert_eq!(result.server, input.server);
        assert_eq!(result.message, input.message);
        assert_eq!(result.procedure, input.procedure);
        assert_eq!(result.class, input.class);
        assert_eq!(result.line, input.line);
        assert_eq!(result.state, input.state);
    }
}
