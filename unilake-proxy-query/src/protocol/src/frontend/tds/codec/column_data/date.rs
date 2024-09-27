use crate::frontend::ColumnData;
use crate::frontend::Result;
use chrono::NaiveDate;
use tokio_util::bytes::{BufMut, BytesMut};
const BASE_DATE: Option<NaiveDate> = NaiveDate::from_ymd_opt(1, 1, 1);

pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) -> Result<()> {
    match data {
        ColumnData::Date(Some(val)) => {
            let base_date = BASE_DATE.unwrap();
            let days_since_base = (*val - base_date).num_days() as u32;

            dst.put_u8(3); // length of the data
            dst.extend_from_slice(&days_since_base.to_le_bytes()[..3]);
        }
        // send null
        _ => {
            dst.put_u8(0);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use tokio_util::bytes::BytesMut;

    use crate::frontend::tds::codec::column_data::date;
    use crate::frontend::{ColumnData, Result};

    const RAW_BYTES: [u8; 4] = [0x03, 0xbb, 0x29, 0x0b];

    #[test]
    fn test_encode_date() -> Result<()> {
        let mut buf = BytesMut::new();
        let data = ColumnData::Date(Some(NaiveDate::from_ymd_opt(2003, 12, 31).unwrap()));

        date::encode(&mut buf, &data)?;

        assert_eq!(buf.to_vec(), RAW_BYTES.to_vec());

        Ok(())
    }
}
