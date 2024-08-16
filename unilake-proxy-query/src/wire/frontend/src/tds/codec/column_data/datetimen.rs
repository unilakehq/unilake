use crate::{ColumnData, DateTime, Error, Result, SmallDateTime};
use tokio_util::bytes::BytesMut;

pub(crate) fn decode(src: &mut BytesMut, len: u8) -> Result<ColumnData> {
    let datetime = match len {
        0 => ColumnData::SmallDateTime(None),
        4 => ColumnData::SmallDateTime(Some(SmallDateTime::decode(src)?)),
        8 => ColumnData::DateTime(Some(DateTime::decode(src)?)),
        _ => {
            return Err(Error::Protocol(
                format!("datetimen: length of {} is invalid", len).into(),
            ))
        }
    };

    Ok(datetime)
}
