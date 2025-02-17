use super::BaseMetaDataColumn;
use crate::frontend::tds::codec::encode;
use crate::frontend::{ColumnData, TdsToken, TdsTokenCodec, TdsTokenType};
use tokio_util::bytes::{BufMut, BytesMut};
use unilake_common::error::TdsWireResult;

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
    fn encode(&self, dest: &mut BytesMut) -> TdsWireResult<()> {
        dest.put_u8(TdsTokenType::ReturnValue as u8);

        dest.put_u16_le(self.param_ordinal);
        encode::write_b_varchar(dest, &self.param_name)?;
        dest.put_u8(self.udf as u8);
        self.meta.encode(dest);
        self.value.encode(dest)?;

        Ok(())
    }

    /// Decode is not implemented for this token type.
    fn decode(_src: &mut BytesMut) -> TdsWireResult<TdsToken> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::{
        ColumnData, DataFlags, FixedLenType, TdsToken, TdsTokenCodec, TdsTokenType,
        TokenReturnValue, TypeInfo,
    };
    use tokio_util::bytes::{Buf, BytesMut};
    use unilake_common::error::TdsWireResult;

    use super::BaseMetaDataColumn;

    #[test]
    fn encode_decode_token_return_value() -> TdsWireResult<()> {
        let input = TokenReturnValue {
            param_ordinal: 0,
            param_name: "some_parm".to_string(),
            udf: false,
            meta: BaseMetaDataColumn {
                flags: DataFlags::default(),
                ty: TypeInfo::FixedLen(FixedLenType::Bit),
            },
            value: ColumnData::BitN(None),
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
