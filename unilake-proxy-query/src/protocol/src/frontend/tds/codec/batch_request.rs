use crate::frontend::error::TdsWireResult;
use crate::frontend::tds::codec::{AllHeaderTy, ALL_HEADERS_LEN_TX};
use crate::frontend::utils::ReadAndAdvance;
use crate::frontend::{Error, TdsMessage, TdsMessageCodec, TokenError};
use byteorder::{ByteOrder, LittleEndian};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

/// SQLBatch Message [2.2.6.7]
#[derive(Debug)]
pub struct BatchRequest {
    pub query: String,
    pub transaction_descriptor: Vec<u8>,
}

impl TdsMessageCodec for BatchRequest {
    fn decode(src: &mut BytesMut) -> TdsWireResult<TdsMessage> {
        let _headers = {
            let mut headers = Vec::with_capacity(2);
            headers.push(src.get_u32_le());
            headers.push(src.get_u32_le() + 4);
            headers
        };

        let _hty = src.get_u16_le();
        let (_, tx_descriptor) = src.read_and_advance(8);
        let _outstanding_requests = src.get_u32_le();

        let qtx: Vec<_> = {
            let max_len = 100_000_000;
            let mut qtx: Vec<u8> = Vec::with_capacity(1024);
            loop {
                let (len, buff) = src.read_and_advance(1024);

                qtx.extend_from_slice(&buff);
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

        Ok(TdsMessage::BatchRequest(BatchRequest {
            query: query_text,
            transaction_descriptor: tx_descriptor.to_vec(),
        }))
    }

    fn encode(&self, dst: &mut BytesMut) -> TdsWireResult<()> {
        dst.put_u32_le(ALL_HEADERS_LEN_TX as u32);
        dst.put_u32_le(ALL_HEADERS_LEN_TX as u32 - 4);
        dst.put_u16_le(AllHeaderTy::TransactionDescriptor as u16);

        dst.put(&self.transaction_descriptor[..]);
        dst.put_u32_le(1);

        for c in self.query.encode_utf16() {
            dst.put_u16_le(c);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::tds::codec::batch_request::BatchRequest;
    use crate::frontend::Result;
    use crate::frontend::TdsMessage;
    use crate::frontend::TdsMessageCodec;
    use tokio_util::bytes::BytesMut;

    #[test]
    fn encode_decode_batchrequest() -> Result<()> {
        let input = BatchRequest {
            query: String::from(
                "SELECT * FROM transactions WHERE transaction = ? AND transaction_descriptor = ?",
            ),
            transaction_descriptor: vec![0; 8],
        };

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).unwrap();

        // decode
        let result = BatchRequest::decode(&mut buff).unwrap();

        // assert
        if let TdsMessage::BatchRequest(result) = result {
            assert_eq!(result.query, input.query);
        } else {
            panic!("unexpected message type: {:?}", result);
        }

        Ok(())
    }
}