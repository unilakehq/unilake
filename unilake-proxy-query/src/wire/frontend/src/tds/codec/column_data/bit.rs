use crate::{ColumnData, Error, Result};
use tokio_util::bytes::{Buf, BytesMut};

/// Zero length token [2.2.4.2.1.1]
pub(crate) fn decode(src: &mut BytesMut) -> Result<ColumnData> {
    let recv_len = src.get_u8() as usize;

    let res = match recv_len {
        0 => ColumnData::Bit(None),
        1 => ColumnData::Bit(Some(src.get_u8() > 0)),
        v => {
            return Err(Error::Protocol(
                format!("bitn: length of {} is invalid", v).into(),
            ))
        }
    };

    Ok(res)
}
