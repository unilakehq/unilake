use crate::{ColumnData, Error, Result};
use tokio_util::bytes::{Buf, BytesMut};

pub(crate) fn decode(src: &mut BytesMut, type_len: usize) -> Result<ColumnData> {
    let len = src.get_u8() as usize;

    let res = match (len, type_len) {
        (0, 4) => ColumnData::F32(None),
        (0, _) => ColumnData::F64(None),
        (4, _) => ColumnData::F32(Some(src.get_f32_le())),
        (8, _) => ColumnData::F64(Some(src.get_f64_le())),
        _ => {
            return Err(Error::Protocol(
                format!("float: length of {} is invalid", len).into(),
            ))
        }
    };

    Ok(res)
}
