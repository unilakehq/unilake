use crate::{ColumnData, DateTimeOffset, Result};
use tokio::io::{AsyncRead, AsyncReadExt};

pub(crate) async fn decode<R>(src: &mut R, len: usize) -> Result<ColumnData<'static>>
where
    R: AsyncRead + Unpin,
{
    let rlen = src.read_u8().await?;

    let dto = match rlen {
        0 => ColumnData::DateTimeOffset(None),
        _ => {
            let dto = DateTimeOffset::decode(src, len, rlen - 5).await?;
            ColumnData::DateTimeOffset(Some(dto))
        }
    };

    Ok(dto)
}
