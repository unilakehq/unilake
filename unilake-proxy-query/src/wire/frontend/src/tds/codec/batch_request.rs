use crate::tds::codec::{AllHeaderTy, ALL_HEADERS_LEN_TX};
use crate::{Error, Result, TokenError};
use byteorder::{ByteOrder, LittleEndian};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// SQLBatch Message [2.2.6.7]
pub struct BatchRequest {
    queries: String,
    transaction_descriptor: [u8; 8],
}

impl BatchRequest {
    pub(crate) async fn decode<R>(src: &mut R) -> Result<BatchRequest>
    where
        R: AsyncRead + Unpin,
    {
        let _headers = {
            let mut headers = Vec::with_capacity(2);
            headers.push(src.read_u32_le().await?);
            headers.push(src.read_u32_le().await? + 4);
            headers
        };

        let _hty = src.read_u16_le().await?;
        let mut tx_descriptor = [0x00u8; 8];
        src.read_exact(&mut tx_descriptor).await?;
        let _outstanding_requests = src.read_u32_le().await?;

        let qtx: Vec<_> = {
            let max_len = 100_000_000;
            let mut qtx: Vec<u8> = Vec::with_capacity(1024);
            loop {
                let mut buff = [0u8; 1024];
                let len = src.read(&mut buff).await?;
                qtx.extend_from_slice(&buff[..len]);
                if len < 1024 {
                    break;
                } else if qtx.len() >= max_len {
                    // TODO: set proper handling of this error and probably any other possible errors
                    return Err(Error::Server(TokenError::new(
                        12,
                        12,
                        12,
                        String::from("QueryTooLong"),
                        String::from("Some Server"),
                        String::from("a"),
                        12,
                    )));
                }
            }
            qtx.chunks(2).map(LittleEndian::read_u16).collect()
        };

        let query_text = String::from_utf16_lossy(&qtx[..]);

        Ok(BatchRequest {
            queries: query_text,
            transaction_descriptor: tx_descriptor,
        })
    }

    pub(crate) async fn encode<W>(&mut self, dst: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dst.write_u32_le(ALL_HEADERS_LEN_TX as u32).await?;
        dst.write_u32_le(ALL_HEADERS_LEN_TX as u32 - 4).await?;
        dst.write_u16_le(AllHeaderTy::TransactionDescriptor as u16)
            .await?;
        dst.write_all(&self.transaction_descriptor).await?;
        dst.write_u32_le(1).await?;

        for c in self.queries.encode_utf16() {
            dst.write_u16_le(c).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tds::codec::batch_request::BatchRequest;
    use crate::Result;
    use tokio::io::{AsyncWriteExt, BufReader, BufWriter};

    #[tokio::test]
    async fn encode_decode_batchrequest() -> Result<()> {
        let mut input = BatchRequest {
            queries: String::from(
                "SELECT * FROM transactions WHERE transaction = ? AND transaction_descriptor = ?",
            ),
            transaction_descriptor: [0x00u8; 8],
        };

        // arrange
        let (inner, outer) = tokio::io::duplex(256);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input.encode(&mut writer).await?;
        writer.flush().await?;

        // decode
        let result = BatchRequest::decode(&mut reader).await?;

        // assert
        assert_eq!(result.queries, input.queries);

        Ok(())
    }
}
