use crate::{ColumnData, Date};
use crate::{Error, Result};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

pub(crate) fn decode(src: &mut BytesMut) -> Result<ColumnData> {
    let len = src.get_u8();

    let res = match len {
        0 => ColumnData::Date(None),
        3 => ColumnData::Date(Some(Date::decode(src)?)),
        _ => {
            return Err(Error::Protocol(
                format!("date: length of {} is invalid", len).into(),
            ))
        }
    };

    Ok(res)
}

pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) -> Result<()> {
    match data {
        ColumnData::Date(Some(val)) => {
            dst.put_u8(3 as u8);
            val.encode(dst);
        }
        _ => {
            dst.put_u8(0 as u8);
        }
    }

    Ok(())
}
