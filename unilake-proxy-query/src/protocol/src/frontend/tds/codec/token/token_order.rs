use crate::frontend::{TdsToken, TdsTokenCodec, TdsTokenType};
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use unilake_common::error::TdsWireResult;

/// Order token [2.2.7.17]
/// Used to inform the client by which columns the data is ordered.
#[allow(dead_code)] // we might want to debug the values
#[derive(Debug)]
pub struct TokenOrder {
    pub column_indexes: Vec<u16>,
}

impl TdsTokenCodec for TokenOrder {
    fn encode(&self, dst: &mut BytesMut) -> TdsWireResult<()> {
        dst.put_u8(TdsTokenType::Order as u8);
        dst.put_u16_le((self.column_indexes.len() * 2) as u16);
        for item in self.column_indexes.iter() {
            dst.put_u16_le(*item);
        }

        Ok(())
    }

    fn decode(src: &mut BytesMut) -> TdsWireResult<TdsToken> {
        let len = src.get_u16_le() / 2;
        let mut column_indexes = Vec::with_capacity(len as usize);
        for _ in 0..len {
            column_indexes.push(src.get_u16_le());
        }

        Ok(TdsToken::Order(TokenOrder { column_indexes }))
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::{TdsToken, TdsTokenCodec, TdsTokenType, TokenOrder};
    use tokio_util::bytes::{Buf, BytesMut};
    use unilake_common::error::TdsWireResult;

    #[test]
    fn encode_decode_token_order() -> TdsWireResult<()> {
        let input = TokenOrder {
            column_indexes: vec![1, 2, 3, 4, 5, 6, 7],
        };

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).expect("this should be ok");

        // decode
        let tokentype = buff.get_u8();
        let result = TokenOrder::decode(&mut buff).unwrap();

        // assert
        assert_eq!(tokentype, TdsTokenType::Order as u8);
        if let TdsToken::Order(result) = result {
            assert_eq!(input.column_indexes, result.column_indexes);
        } else {
            panic!("Did not receive Order Token")
        }
        Ok(())
    }
}
