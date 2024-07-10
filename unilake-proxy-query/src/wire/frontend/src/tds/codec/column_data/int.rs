use crate::{ColumnData, Result};
use tokio_util::bytes::{Buf, BytesMut};

pub(crate) fn decode(src: &mut BytesMut, type_len: usize) -> Result<ColumnData<'static>> {
    let recv_len = src.get_u8() as usize;

    let res = match (recv_len, type_len) {
        (0, 1) => ColumnData::U8(None),
        (0, 2) => ColumnData::I16(None),
        (0, 4) => ColumnData::I32(None),
        (0, _) => ColumnData::I64(None),
        (1, _) => ColumnData::U8(Some(src.get_u8())),
        (2, _) => ColumnData::I16(Some(src.get_i16_le())),
        (4, _) => ColumnData::I32(Some(src.get_i32_le())),
        (8, _) => ColumnData::I64(Some(src.get_i64_le())),
        _ => unimplemented!(),
    };

    Ok(res)
}
