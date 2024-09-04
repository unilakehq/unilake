use crate::frontend::ColumnData;
use crate::frontend::Result;
use tokio_util::bytes::{BufMut, BytesMut};

pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) -> Result<()> {
    match data {
        ColumnData::Date(Some(val)) => {
            // dst.put_u8(3 as u8);
            // val.encode(dst)?;
            todo!()
        }
        // send null
        _ => {
            dst.put_u8(0);
        }
    }

    Ok(())
}
