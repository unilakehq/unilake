use crate::frontend::ColumnData;
use tokio_util::bytes::{BufMut, BytesMut};
use unilake_common::error::TdsWireResult;

/// Fixed length token [2.2.4.2.1.2]
// todo(mrhamburg): remove result type, we are not responding with any errors
pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) -> TdsWireResult<()> {
    match data {
        ColumnData::Bit(val) => {
            dst.put_u8(*val as u8);
        }
        ColumnData::U8(val) => {
            dst.put_u8(*val);
        }
        ColumnData::I16(val) => {
            dst.put_i16_le(*val);
        }
        ColumnData::I32(val) => {
            dst.put_i32_le(*val);
        }
        ColumnData::I64(val) => {
            dst.put_i64_le(*val);
        }
        ColumnData::F32(val) => {
            dst.put_f32_le(*val);
        }
        ColumnData::F64(val) => {
            dst.put_f64_le(*val);
        }
        _ => unreachable!(),
    }

    Ok(())
}
