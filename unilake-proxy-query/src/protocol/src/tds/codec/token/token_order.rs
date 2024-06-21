use crate::{Result, TokenType};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Order token [2.2.7.17]
/// Used to inform the client by which columns the data is ordered.
#[allow(dead_code)] // we might want to debug the values
#[derive(Debug)]
pub struct TokenOrder {
    pub column_indexes: Vec<u16>,
}

impl TokenOrder {
    pub async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let len = src.read_u16_le().await? / 2;
        let mut column_indexes = Vec::with_capacity(len as usize);
        for _ in 0..len {
            column_indexes.push(src.read_u16_le().await?);
        }

        Ok(TokenOrder { column_indexes })
    }

    pub async fn encode<W>(&mut self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TokenType::Order as u8).await?;
        dest.write_u16_le((self.column_indexes.len() * 2) as u16)
            .await?;
        for item in self.column_indexes.iter() {
            dest.write_u16_le(*item).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Result, TokenOrder, TokenType};
    use enumflags2::BitFlags;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

    #[tokio::test]
    async fn encode_decode_token_order() -> Result<()> {
        let mut input = TokenOrder {
            column_indexes: vec![1, 2, 3, 4, 5, 6, 7],
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
        let result = TokenOrder::decode(&mut reader).await?;

        // assert
        assert_eq!(tokentype, TokenType::Order as u8);
        assert_eq!(input.column_indexes, result.column_indexes);

        Ok(())
    }
}
