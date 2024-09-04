use crate::frontend::{Date, DateTime, DateTime2, DateTimeOffset, Result, SmallDateTime, Time};
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use decimal::Decimal;
use sqlstring::SqlString;
use tokio_util::bytes::{BufMut, BytesMut};

use super::TdsTokenCodec;

mod date;
mod datetime2;
pub mod decimal;
mod fixed_len;
mod plp;
pub mod sqlstring;

/// Token definition [2.2.4.2.1]
/// A container of a value that can be represented as a TDS value.
#[derive(Debug)]
pub enum ColumnData {
    /// 8-bit integer, unsigned.
    U8(Option<u8>),
    /// 16-bit integer, signed.
    I16(Option<i16>),
    /// 32-bit integer, signed.
    I32(Option<i32>),
    /// 64-bit integer, signed.
    I64(Option<i64>),
    /// 32-bit floating point number.
    F32(Option<f32>),
    /// 64-bit floating point number.
    F64(Option<f64>),
    /// Boolean.
    Bit(Option<bool>),
    /// A string value.
    String(SqlString),
    /// Binary data.
    Binary(Option<String>),
    /// Numeric value (a decimal).
    Numeric(Option<BigDecimal>),
    /// DateTime value.
    DateTime(Option<NaiveDateTime>),
    /// A small DateTime value.
    SmallDateTime(Option<NaiveDateTime>),
    /// Time value.
    Time(Option<NaiveTime>),
    /// Date value.
    Date(Option<NaiveDate>),
    /// DateTime2 value.
    DateTime2(Option<NaiveDateTime>),
    /// DateTime2 value with an offset.
    DateTimeOffset(Option<NaiveDateTime>),
}

impl ColumnData {
    pub fn type_name(&self) -> String {
        match self {
            ColumnData::U8(_) => "tinyint".into(),
            ColumnData::I16(_) => "smallint".into(),
            ColumnData::I32(_) => "int".into(),
            ColumnData::I64(_) => "bigint".into(),
            ColumnData::F32(_) => "float(24)".into(),
            ColumnData::F64(_) => "float(53)".into(),
            ColumnData::Bit(_) => "bit".into(),
            ColumnData::String(_) => "nvarchar(max)".into(),
            ColumnData::Binary(Some(ref b)) if b.len() <= 8000 => "varbinary(8000)".into(),
            ColumnData::Binary(_) => "varbinary(max)".into(),
            ColumnData::Numeric(Some(ref n)) => {
                format!("numeric({},{})", n.digits(), n.fractional_digit_count()).into()
            }
            ColumnData::Numeric(None) => "numeric".into(),
            ColumnData::DateTime(_) => "datetime".into(),
            ColumnData::SmallDateTime(_) => "smalldatetime".into(),
            ColumnData::Time(_) => "time".into(),
            ColumnData::Date(_) => "date".into(),
            ColumnData::DateTime2(_) => "datetime2".into(),
            ColumnData::DateTimeOffset(_) => "datetimeoffset".into(),
        }
    }

    /// Encode this value into the given destination buffer.
    pub fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        match self {
            ColumnData::Bit(_)
            | ColumnData::U8(_)
            | ColumnData::I16(_)
            | ColumnData::I32(_)
            | ColumnData::I64(_)
            | ColumnData::F32(_)
            | ColumnData::F64(_) => fixed_len::encode(dest, &self)?,
            // todo(mhramburg): would be better if we have the type length from the actual metadata, might also need it for numeric
            ColumnData::String(s) => s.encode(dest)?,
            ColumnData::Date(_) => date::encode(dest, &self)?,
            ColumnData::DateTime2(_) => datetime2::encode(dest, &self)?,
            ColumnData::Numeric(n) => {
                if let Some(n) = n {
                    n.encode(dest)?
                } else {
                    // send null
                    dest.put_u8(0);
                }
            }
            //todo(mhramburg): json, array, bitmap, HLL
            _ => unreachable!("ColumData of type {:?} is not supported", self),
        }

        Ok(())
    }
}
