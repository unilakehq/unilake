use crate::{ColumnData, Result, Time};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

pub(crate) fn decode(src: &mut BytesMut, len: usize) -> Result<ColumnData<'static>> {
    let rlen = src.get_u8();

    let time = match rlen {
        0 => ColumnData::Time(None),
        _ => {
            let time = Time::decode(src, len as usize, rlen as usize)?;
            ColumnData::Time(Some(time))
        }
    };

    Ok(time)
}

pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData<'_>) -> Result<()> {
    match data {
        ColumnData::Time(Some(val)) => {
            dst.put_u8(val.len()? as u8);
            val.encode(dst)?;
        }
        _ => {
            dst.put_u8(0);
        }
    }

    Ok(())
}
