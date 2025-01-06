use crate::frontend::TdsTokenType;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use unilake_common::error::TdsWireResult;

/// Return Status token [2.2.7.18]
/// Used to send the status value of an RPC to the client.
/// The server also uses this token to send the result status value of a T-SQL EXEC query.
pub struct TokenReturnStatus {
    value: i32,
}

impl TokenReturnStatus {
    pub async fn decode<R>(src: &mut R) -> TdsWireResult<Self>
    where
        R: AsyncRead + Unpin,
    {
        Ok(TokenReturnStatus {
            value: src.read_i32_le().await?,
        })
    }

    pub async fn encode<W>(&mut self, dest: &mut W) -> TdsWireResult<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TdsTokenType::ReturnStatus as u8).await?;
        dest.write_i32_le(self.value).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::{TdsTokenType, TokenReturnStatus};
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
    use unilake_common::error::TdsWireResult;

    #[tokio::test]
    async fn encode_decode_token_return_status() -> TdsWireResult<()> {
        let mut input = TokenReturnStatus { value: 12 };

        // arrange
        let (inner, outer) = tokio::io::duplex(256);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input.encode(&mut writer).await?;
        writer.flush().await?;

        // decode
        let tokentype = reader.read_u8().await?;
        let result = TokenReturnStatus::decode(&mut reader).await?;

        // assert
        assert_eq!(tokentype, TdsTokenType::ReturnStatus as u8);
        assert_eq!(input.value, result.value);

        Ok(())
    }
}
