use crate::{ColumnData, Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt};

pub(crate) async fn decode<R>(src: &mut R, type_len: usize) -> Result<ColumnData<'static>>
where
    R: AsyncRead + Unpin,
{
    let len = src.read_u8().await? as usize;

    let res = match (len, type_len) {
        (0, 4) => ColumnData::F32(None),
        (0, _) => ColumnData::F64(None),
        (4, _) => ColumnData::F32(Some(src.read_f32_le().await?)),
        (8, _) => ColumnData::F64(Some(src.read_f64_le().await?)),
        _ => {
            return Err(Error::Protocol(
                format!("float: length of {} is invalid", len).into(),
            ))
        }
    };

    Ok(res)
}
