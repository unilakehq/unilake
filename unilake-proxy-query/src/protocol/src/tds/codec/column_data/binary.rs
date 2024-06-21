use crate::{ColumnData, Result};
use std::borrow::Cow;
use tokio::io::AsyncRead;

pub(crate) async fn decode<R>(src: &mut R, len: usize) -> Result<ColumnData<'static>>
where
    R: AsyncRead + Unpin,
{
    let data = super::plp::decode(src, len).await?.map(Cow::from);

    Ok(ColumnData::Binary(data))
}
