use crate::tds::codec::{decode, encode};
use crate::{Result, TokenType};
use std::mem::size_of;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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
    pub async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let _length = src.read_u16_le().await?;

        let number = src.read_u32_le().await?;
        let state = src.read_u8().await?;
        let class = src.read_u8().await?;
        let message = decode::read_us_varchar(src).await?;
        let server = decode::read_b_varchar(src).await?;
        let procedure = decode::read_b_varchar(src).await?;
        let line = src.read_u32_le().await?;

        Ok(TokenInfo {
            number,
            state,
            class,
            message,
            server,
            procedure,
            line,
        })
    }

    pub async fn encode<W>(&mut self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TokenType::Info as u8).await?;

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

        dest.write_u16_le(length).await?;
        dest.write_u32_le(self.number).await?;
        dest.write_u8(self.state).await?;
        dest.write_u8(self.class).await?;
        encode::write_us_varchar(dest, &self.message).await?;
        encode::write_b_varchar(dest, &self.server).await?;
        encode::write_b_varchar(dest, &self.procedure).await?;
        dest.write_u32_le(self.line).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

    use crate::{Result, TokenInfo, TokenType};

    #[tokio::test]
    async fn encode_decode_token_info() -> Result<()> {
        let mut input = TokenInfo {
            line: 12,
            class: 1,
            number: 1321,
            message: String::from("Hello World"),
            state: 2,
            procedure: String::from("sp.my_proc"),
            server: String::from("mydatabase"),
        };

        // arrange
        let (inner, outer) = tokio::io::duplex(256);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input.encode(&mut writer).await?;
        writer.flush().await?;

        // decode
        let token_type = reader.read_u8().await?;
        let result = TokenInfo::decode(&mut reader).await?;

        // assert
        assert_eq!(token_type, TokenType::Info as u8);
        assert_eq!(result.server, input.server);
        assert_eq!(result.message, input.message);
        assert_eq!(result.procedure, input.procedure);
        assert_eq!(result.class, input.class);
        assert_eq!(result.line, input.line);
        assert_eq!(result.state, input.state);
        assert_eq!(result.number, input.number);

        Ok(())
    }
}
