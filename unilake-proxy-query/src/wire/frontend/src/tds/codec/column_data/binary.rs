use crate::{ColumnData, Result};
use tokio_util::bytes::{BufMut, BytesMut};

pub(crate) fn decode(src: &mut BytesMut, len: usize) -> Result<ColumnData> {
    // todo(mrhamburg): fix this
    todo!()
    // let data = super::plp::decode(src, len)?.map(String::try_from);

    // Ok(ColumnData::Binary(data))
}

pub(crate) fn encode(dst: &mut BytesMut, data: &[u8]) -> Result<()> {
    dst.put_slice(data);
    Ok(())
}
