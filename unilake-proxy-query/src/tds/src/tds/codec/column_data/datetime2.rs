use crate::{ColumnData, DateTime2, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub(crate) async fn decode<R>(src: &mut R, len: usize) -> Result<ColumnData<'static>>
where
    R: AsyncRead + Unpin,
{
    let rlen = src.read_u8().await?;

    let date = match rlen {
        0 => ColumnData::DateTime2(None),
        rlen => {
            let dt = DateTime2::decode(src, len, rlen as usize - 3).await?;
            ColumnData::DateTime2(Some(dt))
        }
    };

    Ok(date)
}

pub(crate) async fn encode<W>(dst: &mut W, data: &ColumnData<'_>) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    match data {
        ColumnData::DateTime2(None) => {
            dst.write_u8(0).await?;
        }
        ColumnData::DateTime2(Some(val)) => {
            dst.write_u8((3 + val.time().len()?) as u8).await?;
            val.encode(dst).await?;
        }
        _ => {}
    }

    Ok(())
}
