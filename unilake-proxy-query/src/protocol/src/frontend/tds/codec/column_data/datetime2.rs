use crate::frontend::{ColumnData, Result};
use tokio_util::bytes::{BufMut, BytesMut};

pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) -> Result<()> {
    match data {
        ColumnData::DateTime2(Some(val)) => {
            dst.put_u8((3 + val.time().len()?) as u8);
            val.encode(dst)?;
        }
        // send null
        _ => dst.put_u8(0),
    }

    Ok(())
}
