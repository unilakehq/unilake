use crate::frontend::tds::codec::column_data::ColumnData;
use tokio_util::bytes::{BufMut, BytesMut};

/// Fixed length token [2.2.4.2.1.2]
pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) {
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
}
