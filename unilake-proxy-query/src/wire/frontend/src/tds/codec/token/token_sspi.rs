use crate::{Result, TdsTokenType};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Sspi token [2.2.7.22]
/// The SSPI token returned during the login process.
#[derive(Debug)]
pub struct TokenSspi(Vec<u8>);

impl TokenSspi {
    pub async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let len = src.read_u16_le().await? as usize;
        let mut bytes = vec![0; len];
        src.read_exact(&mut bytes[0..len]).await?;

        Ok(Self(bytes))
    }

    pub async fn encode<W>(&mut self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TdsTokenType::Sspi as u8).await?;
        dest.write_u16_le(self.0.len() as u16).await?;
        dest.write_all(&self.0).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Result, TokenSspi, TdsTokenType};
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

    #[tokio::test]
    async fn encode_decode_token_sspi() -> Result<()> {
        let mut input = TokenSspi {
            0: vec![1, 2, 3, 4, 5, 6, 7],
        };

        // arrange
        let (inner, outer) = tokio::io::duplex(256);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input.encode(&mut writer).await?;
        writer.flush().await?;

        // decode
        let tokentype = reader.read_u8().await?;
        let result = TokenSspi::decode(&mut reader).await?;

        // assert
        assert_eq!(tokentype, TdsTokenType::Sspi as u8);
        assert_eq!(input.0, result.0);

        Ok(())
    }
}
