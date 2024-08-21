use super::BaseMetaDataColumn;
use crate::tds::codec::{decode, encode};
use crate::{ColumnData, Error, Result, TdsToken, TdsTokenCodec, TdsTokenType};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

/// ReturnValue Token [2.2.7.19]
/// Used to send the return value of an RPC to the client. When an RPC is executed,
/// the associated parameters might be defined as input or output (or "return") parameters.
/// This token is used to send a description of the return parameter to the client. This token is
/// also used to describe the value returned by a UDF when executed as an RPC.
#[derive(Debug)]
pub struct TokenReturnValue {
    pub param_ordinal: u16,
    pub param_name: String,
    /// return value of user defined function
    pub udf: bool,
    pub meta: BaseMetaDataColumn,
    pub value: ColumnData,
}

impl TdsTokenCodec for TokenReturnValue {
    /// Decode is not implemented for this token type.
    fn decode(src: &mut BytesMut) -> Result<TdsToken> {
        unimplemented!()
    }

    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::ReturnValue as u8);

        dest.put_u16_le(self.param_ordinal);
        encode::write_b_varchar(dest, &self.param_name)?;
        dest.put_u8(self.udf as u8);
        self.meta.encode(dest)?;
        self.value.encode(dest)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use enumflags2::BitFlags;
    use tokio_util::bytes::{Buf, BytesMut};

    use crate::{
        ColumnData, FixedLenType, Result, TdsToken, TdsTokenCodec, TdsTokenType, TokenReturnValue,
        TypeInfo,
    };

    use super::BaseMetaDataColumn;

    #[test]
    fn encode_decode_token_return_value() -> Result<()> {
        let input = TokenReturnValue {
            param_ordinal: 0,
            param_name: "some_parm".to_string(),
            udf: false,
            meta: BaseMetaDataColumn {
                flags: BitFlags::empty(),
                ty: TypeInfo::FixedLen(FixedLenType::Bit),
            },
            value: ColumnData::Bit(None),
        };

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).expect("this should be ok");

        // decode
        let tokentype = buff.get_u8();
        let result = TokenReturnValue::decode(&mut buff).unwrap();

        // assert
        assert_eq!(tokentype, TdsTokenType::ReturnValue as u8);
        if let TdsToken::ReturnValue(result) = result {
            assert_eq!(input.param_ordinal, result.param_ordinal);
            assert_eq!(input.param_name, result.param_name);
            assert_eq!(input.udf, result.udf);

        //assert_eq!(input.meta.ty, result.meta.ty);
        //assert_eq!(input.meta.flags, result.meta.flags);
        //assert_eq!(input.value, result.value);
        } else {
            panic!("Could not find Return Value Token")
        }
        Ok(())
    }
}
