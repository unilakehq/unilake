use crate::{ColumnData, DateTime, Error, Result, SmallDateTime};
use tokio::io::AsyncRead;

pub(crate) async fn decode<R>(src: &mut R, len: u8) -> Result<ColumnData<'static>>
where
    R: AsyncRead + Unpin,
{
    let datetime = match len {
        0 => ColumnData::SmallDateTime(None),
        4 => ColumnData::SmallDateTime(Some(SmallDateTime::decode(src).await?)),
        8 => ColumnData::DateTime(Some(DateTime::decode(src).await?)),
        _ => {
            return Err(Error::Protocol(
                format!("datetimen: length of {} is invalid", len).into(),
            ))
        }
    };

    Ok(datetime)
}
