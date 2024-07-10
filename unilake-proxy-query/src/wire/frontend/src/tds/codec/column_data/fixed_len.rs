use crate::{ColumnData, FixedLenType, Result};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

/// Fixed length token [2.2.4.2.1.2]
pub(crate) fn decode(src: &mut BytesMut, r#type: &FixedLenType) -> Result<ColumnData<'static>> {
    let data = match r#type {
        FixedLenType::Null => ColumnData::Bit(None),
        FixedLenType::Bit => ColumnData::Bit(Some(src.get_u8() != 0)),
        FixedLenType::Int1 => ColumnData::U8(Some(src.get_u8())),
        FixedLenType::Int2 => ColumnData::I16(Some(src.get_i16_le())),
        FixedLenType::Int4 => ColumnData::I32(Some(src.get_i32_le())),
        FixedLenType::Int8 => ColumnData::I64(Some(src.get_i64_le())),
        FixedLenType::Float4 => ColumnData::F32(Some(src.get_f32_le())),
        FixedLenType::Float8 => ColumnData::F64(Some(src.get_f64_le())),
        //FixedLenType::Datetime => super::datetimen::decode(src, 8).await?,
        //FixedLenType::Datetime4 => super::datetimen::decode(src, 4).await?,
        _ => todo!(),
    };

    Ok(data)
}

pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData<'_>) -> Result<()> {
    match data {
        ColumnData::Bit(Some(val)) => {
            dst.put_u8(*val as u8);
        }
        ColumnData::U8(Some(val)) => {
            dst.put_u8(*val);
        }
        ColumnData::I16(Some(val)) => {
            dst.put_i16_le(*val);
        }
        ColumnData::I32(Some(val)) => {
            dst.put_i32_le(*val);
        }
        ColumnData::I64(Some(val)) => {
            dst.put_i64_le(*val);
        }
        ColumnData::F32(Some(val)) => {
            dst.put_f32_le(*val);
        }
        ColumnData::F64(Some(val)) => {
            dst.put_f64_le(*val);
        }
        _ => {
            unreachable!("Unsupported column type: {:?}", data);
        }
    }

    Ok(())
}
