use crate::{ColumnData, Result};
use tokio_util::bytes::BytesMut;

pub(crate) fn encode(dest: &mut BytesMut, type_length: &usize, data: &ColumnData) -> Result<()> {
    match data {
        ColumnData::String(str) => {
            if let Some(str) = str {
                super::plp::encode(dest, type_length, Some(str));
            } else {
                super::plp::encode(dest, type_length, None);
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
