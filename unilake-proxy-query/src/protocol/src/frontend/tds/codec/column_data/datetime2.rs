use crate::frontend::ColumnData;
use chrono::{NaiveDate, Timelike};
use tokio_util::bytes::{BufMut, BytesMut};
use unilake_common::error::TdsWireResult;

const BASE_DATE: Option<NaiveDate> = NaiveDate::from_ymd_opt(1, 1, 1);

pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) -> TdsWireResult<()> {
    match data {
        ColumnData::DateTime2(Some(val)) => {
            // todo(mrhamburg): we currently always assume a scale of 7 (microseconds)
            let scale = 7;

            // Determine how many bytes are needed based on the scale
            let time_bytes = match scale {
                0..=2 => 3,
                3..=4 => 4,
                5..=7 => 5,
                _ => unreachable!(),
            };
            dst.put_u8((3 + time_bytes) as u8);

            // set time
            let time_of_day_ns = (val.time().num_seconds_from_midnight() as u64 * 1_000_000_000)
                + (val.time().nanosecond() as u64);

            let scaled_time: u64 = match scale {
                1 => 100000000,
                2 => 10000000,
                3 => 1000000,
                4 => 100000,
                5 => 10000,
                6 => 1000,
                7 => 100,
                _ => unreachable!(),
            };

            let scaled_time = time_of_day_ns / scaled_time;

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
    use crate::frontend::{tds::codec::column_data::datetime2, ColumnData};
    use chrono::NaiveDate;
    use tokio_util::bytes::BytesMut;
    use unilake_common::error::TdsWireResult;

    // todo(mrhamburg): we need this to also properly work with scale 6 (Max for StarRocks afaik)!
    const RAW_BYTES_SCALE_7: [u8; 9] = [0x08, 0x80, 0xb7, 0x14, 0xab, 0x08, 0xbb, 0x29, 0x0b];

    #[test]
    fn test_encode_datetime2() -> TdsWireResult<()> {
        let mut buf = BytesMut::new();
        let data = ColumnData::DateTime2(Some(
            NaiveDate::from_ymd_opt(2003, 12, 31)
                .unwrap()
                .and_hms_milli_opt(01, 02, 03, 0)
                .unwrap(),
        ));

        datetime2::encode(&mut buf, &data)?;

        assert_eq!(buf.to_vec(), RAW_BYTES_SCALE_7.to_vec());

        Ok(())
    }
}
