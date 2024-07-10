use crate::{ColumnData, Result};
use std::borrow::Cow;
use tokio_util::bytes::BytesMut;

pub(crate) fn decode(src: &mut BytesMut, len: usize) -> Result<ColumnData<'static>> {
    let data = super::plp::decode(src, len)?.map(Cow::from);

    Ok(ColumnData::Binary(data))
}
