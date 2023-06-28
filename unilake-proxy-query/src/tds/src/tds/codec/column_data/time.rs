use crate::{ColumnData, Result, Time};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub(crate) async fn decode<R>(src: &mut R, len: usize) -> Result<ColumnData<'static>>
where
    R: AsyncRead + Unpin,
{
    let rlen = src.read_u8().await?;

    let time = match rlen {
        0 => ColumnData::Time(None),
        _ => {
            let time = Time::decode(src, len as usize, rlen as usize).await?;
            ColumnData::Time(Some(time))
        }
    };

    Ok(time)
}

pub(crate) async fn encode<W>(dst: &mut W, data: &ColumnData<'_>) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    match data {
        ColumnData::Time(Some(val)) => {
            dst.write_u8(val.len()? as u8).await?;
            val.encode(dst).await?;
        }
        _ => {
            dst.write_u8(0).await?;
        }
    }

    Ok(())
}
