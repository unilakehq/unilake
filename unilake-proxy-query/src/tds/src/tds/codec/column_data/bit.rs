use crate::{ColumnData, Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt};

/// Zero length token [2.2.4.2.1.1]
pub(crate) async fn decode<R>(src: &mut R) -> Result<ColumnData<'static>>
where
    R: AsyncRead + Unpin,
{
    let recv_len = src.read_u8().await? as usize;

    let res = match recv_len {
        0 => ColumnData::Bit(None),
        1 => ColumnData::Bit(Some(src.read_u8().await? > 0)),
        v => {
            return Err(Error::Protocol(
                format!("bitn: length of {} is invalid", v).into(),
            ))
        }
    };

    Ok(res)
}
