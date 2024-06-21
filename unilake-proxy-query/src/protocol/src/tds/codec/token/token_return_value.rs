use super::BaseMetaDataColumn;
use crate::tds::codec::{decode, encode};
use crate::{ColumnData, Error, Result, TokenType};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// ReturnValue Token [2.2.7.19]
/// Used to send the return value of an RPC to the client. When an RPC is executed,
/// the associated parameters might be defined as input or output (or "return") parameters.
/// This token is used to send a description of the return parameter to the client. This token is
/// also used to describe the value returned by a UDF when executed as an RPC.
pub struct TokenReturnValue {
    pub param_ordinal: u16,
    pub param_name: String,
    /// return value of user defined function
    pub udf: bool,
    pub meta: BaseMetaDataColumn,
    pub value: ColumnData<'static>,
}

impl TokenReturnValue {
    pub async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let param_ordinal = src.read_u16_le().await?;
        let param_name = decode::read_b_varchar(src).await?;

        let udf = match src.read_u8().await? {
            0x01 => false,
            0x02 => true,
            _ => return Err(Error::Protocol("ReturnValue: invalid status".into())),
        };

        let meta = BaseMetaDataColumn::decode(src).await?;
        let value = ColumnData::decode(src, &meta.ty).await?;

        let token = TokenReturnValue {
            param_ordinal,
            param_name,
            udf,
            meta,
            value,
        };

        Ok(token)
    }

    pub async fn encode<W>(&mut self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TokenType::ReturnValue as u8).await?;

        dest.write_u16_le(self.param_ordinal).await?;
        encode::write_b_varchar(dest, &self.param_name).await?;
        dest.write_u8(self.udf as u8).await?;
        self.meta.encode(dest).await?;
        self.value.encode(dest).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use enumflags2::BitFlags;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

    use crate::{ColumnData, FixedLenType, Result, TokenReturnValue, TokenType, TypeInfo};

    use super::BaseMetaDataColumn;

    #[tokio::test]
    async fn encode_decode_token_return_value() -> Result<()> {
        let mut input = TokenReturnValue {
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
        let (inner, outer) = tokio::io::duplex(256);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input.encode(&mut writer).await?;
        writer.flush().await?;

        // decode
        let tokentype = reader.read_u8().await?;
        let result = TokenReturnValue::decode(&mut reader).await?;

        // assert
        assert_eq!(tokentype, TokenType::ReturnValue as u8);
        assert_eq!(input.param_ordinal, result.param_ordinal);
        assert_eq!(input.param_name, result.param_name);
        assert_eq!(input.udf, result.udf);
        //assert_eq!(input.meta.ty, result.meta.ty);
        //assert_eq!(input.meta.flags, result.meta.flags);
        //assert_eq!(input.value, result.value);

        Ok(())
    }
}
