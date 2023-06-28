use crate::{ColumnData, FixedLenType, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Fixed length token [2.2.4.2.1.2]
pub(crate) async fn decode<R>(src: &mut R, r#type: &FixedLenType) -> Result<ColumnData<'static>>
where
    R: AsyncRead + Unpin,
{
    let data = match r#type {
        FixedLenType::Null => ColumnData::Bit(None),
        FixedLenType::Bit => ColumnData::Bit(Some(src.read_u8().await? != 0)),
        FixedLenType::Int1 => ColumnData::U8(Some(src.read_u8().await?)),
        FixedLenType::Int2 => ColumnData::I16(Some(src.read_i16_le().await?)),
        FixedLenType::Int4 => ColumnData::I32(Some(src.read_i32_le().await?)),
        FixedLenType::Int8 => ColumnData::I64(Some(src.read_i64_le().await?)),
        FixedLenType::Float4 => ColumnData::F32(Some(src.read_f32_le().await?)),
        FixedLenType::Float8 => ColumnData::F64(Some(src.read_f64_le().await?)),
        //FixedLenType::Datetime => super::datetimen::decode(src, 8).await?,
        //FixedLenType::Datetime4 => super::datetimen::decode(src, 4).await?,
        _ => todo!(),
    };

    Ok(data)
}

pub(crate) async fn encode<W>(dst: &mut W, data: &ColumnData<'_>) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    match data {
        ColumnData::Bit(Some(val)) => {
            dst.write_u8(*val as u8).await?;
        }
        ColumnData::U8(Some(val)) => {
            dst.write_u8(*val).await?;
        }
        ColumnData::I16(Some(val)) => {
            dst.write_i16_le(*val).await?;
        }
        ColumnData::I32(Some(val)) => {
            dst.write_i32_le(*val).await?;
        }
        ColumnData::I64(Some(val)) => {
            dst.write_i64_le(*val).await?;
        }
        ColumnData::F32(Some(val)) => {
            dst.write_f32_le(*val).await?;
        }
        ColumnData::F64(Some(val)) => {
            dst.write_f64_le(*val).await?;
        }
        _ => {
            unreachable!("Unsupported column type: {:?}", data);
        }
    }

    Ok(())
}
