use crate::{ColumnData, Date};
use crate::{Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub(crate) async fn decode<R>(src: &mut R) -> Result<ColumnData<'static>>
where
    R: AsyncRead + Unpin,
{
    let len = src.read_u8().await?;

    let res = match len {
        0 => ColumnData::Date(None),
        3 => ColumnData::Date(Some(Date::decode(src).await?)),
        _ => {
            return Err(Error::Protocol(
                format!("date: length of {} is invalid", len).into(),
            ))
        }
    };

    Ok(res)
}

pub(crate) async fn encode<W>(dst: &mut W, data: &ColumnData<'_>) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    match data {
        ColumnData::Date(Some(val)) => {
            dst.write_u8(3 as u8).await?;
            val.encode(dst);
        }
        _ => {
            dst.write_u8(0 as u8).await?;
        }
    }

    Ok(())
}
