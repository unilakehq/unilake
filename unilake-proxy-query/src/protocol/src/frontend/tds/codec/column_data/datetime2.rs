use crate::frontend::{ColumnData, Result};
use bigdecimal::num_traits::ToBytes;
use chrono::{NaiveDate, Timelike};
use tokio_util::bytes::{BufMut, BytesMut};

const BASE_DATE: Option<NaiveDate> = NaiveDate::from_ymd_opt(1, 1, 1);

pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) -> Result<()> {
    match data {
        ColumnData::DateTime2(Some(val)) => {
            let scale = 5;
            dst.put_u8((3 + scale) as u8);

            // set time
            let nanoseconds_per_day = 24 * 60 * 60 * 1_000_000_000;
            let time_of_day_ns = (val.time().num_seconds_from_midnight() as u64 * 1_000_000_000)
                + (val.time().nanosecond() as u64);

            let max_fractional_units = 10_u64.pow(scale as u32);
            let scaled_time = (time_of_day_ns * max_fractional_units) / nanoseconds_per_day;

            // Determine how many bytes are needed based on the scale
            let time_bytes = match scale {
                0..=2 => 3,
                3..=4 => 4,
                5..=7 => 5,
                _ => unreachable!(),
            };

            // Extract the necessary bytes for the time component
            dst.extend_from_slice(&scaled_time.to_le_bytes()[..time_bytes]);

            // set date
            let base_date = BASE_DATE.unwrap();
            let days_since_base = (val.date() - base_date).num_days() as u32;

            dst.extend_from_slice(&days_since_base.to_le_bytes()[..3]);
        }
        // send null
        _ => dst.put_u8(0),
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use chrono::NaiveDate;
    use tokio_util::bytes::BytesMut;

    use crate::frontend::{tds::codec::column_data::datetime2, ColumnData, Result};

    const RAW_BYTES: [u8; 9] = [0x08, 0xa0, 0x97, 0xff, 0x8f, 0xab, 0x3c, 0x47, 0x0b];

    #[test]
    fn test_encode_datetime2() -> Result<()> {
        let mut buf = BytesMut::new();
        let data = ColumnData::DateTime2(Some(
            NaiveDate::from_ymd_opt(2024, 9, 4)
                .unwrap()
                .and_hms_milli_opt(20, 28, 5, 0)
                .unwrap(),
        ));

        datetime2::encode(&mut buf, &data)?;

        assert_eq!(buf.to_vec(), RAW_BYTES.to_vec());

        Ok(())
    }
}
