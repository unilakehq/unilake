use crate::{ColumnData, DateTimeOffset, Result};
use tokio_util::bytes::{Buf, BytesMut};

pub(crate) fn decode(src: &mut BytesMut, len: usize) -> Result<ColumnData<'static>> {
    let rlen = src.get_u8();

    let dto = match rlen {
        0 => ColumnData::DateTimeOffset(None),
        _ => {
            let dto = DateTimeOffset::decode(src, len, rlen - 5)?;
            ColumnData::DateTimeOffset(Some(dto))
        }
    };

    Ok(dto)
}
