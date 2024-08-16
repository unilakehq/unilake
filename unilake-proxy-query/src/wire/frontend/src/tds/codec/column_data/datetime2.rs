use crate::{ColumnData, DateTime2, Result};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

pub(crate) fn decode(src: &mut BytesMut, len: usize) -> Result<ColumnData> {
    let rlen = src.get_u8();

    let date = match rlen {
        0 => ColumnData::DateTime2(None),
        rlen => {
            let dt = DateTime2::decode(src, len, rlen as usize - 3)?;
            ColumnData::DateTime2(Some(dt))
        }
    };

    Ok(date)
}

pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) -> Result<()> {
    match data {
        ColumnData::DateTime2(None) => {
            dst.put_u8(0);
        }
        ColumnData::DateTime2(Some(val)) => {
            dst.put_u8((3 + val.time().len()?) as u8);
            val.encode(dst)?;
        }
        _ => {}
    }

    Ok(())
}
