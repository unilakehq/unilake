use crate::frontend::{ColumnData, Result};
use tokio_util::bytes::{BufMut, BytesMut};

/// Variable length token [2.2.4.2.1.3]
pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) -> Result<()> {
    match data {
        ColumnData::BitN(Some(val)) => {
            dst.put_u8(1);
            dst.put_u8(*val as u8);
        }
        ColumnData::U8N(Some(val)) => {
            dst.put_u8(1);
            dst.put_u8(*val);
        }
        ColumnData::I16N(Some(val)) => {
            dst.put_u8(2);
            dst.put_i16_le(*val);
        }
        ColumnData::I32N(Some(val)) => {
            dst.put_u8(4);
            dst.put_i32_le(*val);
        }
        ColumnData::I64N(Some(val)) => {
            dst.put_u8(8);
            dst.put_i64_le(*val);
        }
        ColumnData::F32N(Some(val)) => {
            dst.put_u8(4);
            dst.put_f32_le(*val);
        }
        ColumnData::F64N(Some(val)) => {
            dst.put_u8(8);
            dst.put_f64_le(*val);
        }
        _ => dst.put_u8(0),
    }

    // push null
    Ok(())
}
