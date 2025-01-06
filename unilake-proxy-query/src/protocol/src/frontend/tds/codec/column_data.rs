use crate::frontend::{TypeInfo, VarLenType};
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlstring::SqlString;
use tokio_util::bytes::BytesMut;
use unilake_common::error::TdsWireResult;

mod date;
mod datetime2;
pub mod decimal;
mod fixed_len;
mod numeric;
mod plp;
pub mod sqlstring;
mod var_len;

/// Token definition [2.2.4.2.1]
/// A container of a value that can be represented as a TDS value.
#[derive(Debug, Clone)]
pub enum ColumnData {
    /// 8-bit integer, unsigned (fixed-length).
    U8(u8),
    /// 8-bit integer, unsigned (var-length).
    U8N(Option<u8>),
    /// 16-bit integer, signed (fixed-length).
    I16(i16),
    /// 16-bit integer, signed (var-length).
    I16N(Option<i16>),
    /// 32-bit integer, signed (fixed-length).
    I32(i32),
    /// 32-bit integer, signed (var-length).
    I32N(Option<i32>),
    /// 64-bit integer, signed (fixed-length).
    I64(i64),
    /// 64-bit integer, signed (var-length).
    I64N(Option<i64>),
    /// 32-bit floating point number (fixed-length).
    F32(f32),
    /// 32-bit floating point number (var-length).
    F32N(Option<f32>),
    /// 64-bit floating point number (fixed-length).
    F64(f64),
    /// 64-bit floating point number (var-length).
    F64N(Option<f64>),
    /// Boolean (fixed-length).
    Bit(bool),
    /// Boolean (var-length).
    BitN(Option<bool>),
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
    pub fn new_varchar(value: &str, max_length: usize) -> Self {
        ColumnData::String(SqlString::from_string(Some(value.to_string()), max_length))
    }

    /// Returns the size of the column data in bytes.
    /// Does not take into account protocol overhead
    pub fn size_in_bytes(&self) -> usize {
        match self {
            ColumnData::U8(_) => 1,
            ColumnData::U8N(v) => {
                if v.is_some() {
                    1
                } else {
                    0
                }
            }
            ColumnData::I16(_) => 2,
            ColumnData::I16N(v) => {
                if v.is_some() {
                    2
                } else {
                    0
                }
            }
            ColumnData::I32(_) => 4,
            ColumnData::I32N(v) => {
                if v.is_some() {
                    4
                } else {
                    0
                }
            }
            ColumnData::I64(_) => 8,
            ColumnData::I64N(v) => {
                if v.is_some() {
                    8
                } else {
                    0
                }
            }
            ColumnData::F32(_) => 4,
            ColumnData::F32N(v) => {
                if v.is_some() {
                    4
                } else {
                    0
                }
            }
            ColumnData::F64(_) => 8,
            ColumnData::F64N(v) => {
                if v.is_some() {
                    8
                } else {
                    0
                }
            }
            ColumnData::Bit(_) => 1,
            ColumnData::BitN(v) => {
                if v.is_some() {
                    1
                } else {
                    0
                }
            }
            ColumnData::String(v) => {
                if !v.is_empty() {
                    v.len() * 2
                } else {
                    0
                }
            }
            ColumnData::Binary(v) => {
                if v.is_some() {
                    v.as_ref().unwrap().len()
                } else {
                    0
                }
            }
            ColumnData::Numeric(v) => {
                if v.is_some() {
                    8
                } else {
                    0
                }
            }
            ColumnData::DateTime(v) => {
                if v.is_some() {
                    8
                } else {
                    0
                }
            }
            ColumnData::SmallDateTime(v) => {
                if v.is_some() {
                    4
                } else {
                    0
                }
            }
            ColumnData::Time(v) => {
                if v.is_some() {
                    4
                } else {
                    0
                }
            }
            ColumnData::Date(v) => {
                if v.is_some() {
                    4
                } else {
                    0
                }
            }
            ColumnData::DateTime2(v) => {
                if v.is_some() {
                    8
                } else {
                    0
                }
            }
            ColumnData::DateTimeOffset(v) => {
                if v.is_some() {
                    8
                } else {
                    0
                }
            }
        }
    }

    /// Encode this value into the given destination buffer.
    pub fn encode(&self, dest: &mut BytesMut) -> TdsWireResult<()> {
        match self {
            ColumnData::Bit(_)
            | ColumnData::U8(_)
            | ColumnData::I16(_)
            | ColumnData::I32(_)
            | ColumnData::I64(_)
            | ColumnData::F32(_)
            | ColumnData::F64(_) => fixed_len::encode(dest, &self)?,
            ColumnData::BitN(_)
            | ColumnData::U8N(_)
            | ColumnData::I16N(_)
            | ColumnData::I32N(_)
            | ColumnData::I64N(_)
            | ColumnData::F32N(_)
            | ColumnData::F64N(_) => var_len::encode(dest, &self)?,
            ColumnData::String(s) => s.encode(dest)?,
            ColumnData::Date(_) => date::encode(dest, &self),
            ColumnData::DateTime2(_) => datetime2::encode(dest, &self)?,
            ColumnData::Numeric(n) => {
                numeric::encode(dest, &n)?;
            }
            _ => unreachable!("ColumData of type {:?} is not supported", self),
        }

        Ok(())
    }

    // todo(mrhamburg): further implement these types
    pub fn decode(src: &mut BytesMut, typeinfo: &TypeInfo) -> TdsWireResult<Self> {
        match typeinfo {
            TypeInfo::FixedLen(fl) => todo!(),
            TypeInfo::VarLenSized(vs) => match vs.r#type() {
                VarLenType::NVarchar => Ok(ColumnData::String(SqlString::decode(src, vs.len())?)),
                _ => todo!(),
            },
            TypeInfo::VarLenSizedPrecision { .. } => todo!(),
        }
    }
}
