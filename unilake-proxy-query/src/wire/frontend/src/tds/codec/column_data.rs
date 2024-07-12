use crate::{
    Date, DateTime, DateTime2, DateTimeOffset, Numeric, Result, SmallDateTime, Time, TypeInfo,
    VarLenType,
};
use std::borrow::Cow;
use tokio_util::bytes::BytesMut;

mod binary;
mod bit;
mod date;
mod datetime2;
mod datetimen;
mod datetimeoffsetn;
mod fixed_len;
mod float;
mod int;
mod plp;
mod string;
mod time;
mod var_len;

const MAX_NVARCHAR_SIZE: usize = 1 << 30;

/// Token definition [2.2.4.2.1]
/// A container of a value that can be represented as a TDS value.
#[derive(Clone, Debug)]
pub enum ColumnData<'a> {
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
    String(Option<Cow<'a, str>>),
    /// Binary data.
    Binary(Option<Cow<'a, [u8]>>),
    /// Numeric value (a decimal).
    Numeric(Option<Numeric>),
    /// DateTime value.
    DateTime(Option<DateTime>),
    /// A small DateTime value.
    SmallDateTime(Option<SmallDateTime>),
    /// Time value.
    Time(Option<Time>),
    /// Date value.
    Date(Option<Date>),
    /// DateTime2 value.
    DateTime2(Option<DateTime2>),
    /// DateTime2 value with an offset.
    DateTimeOffset(Option<DateTimeOffset>),
}

impl<'a> ColumnData<'a> {
    pub fn type_name(&self) -> Cow<'static, str> {
        match self {
            ColumnData::U8(_) => "tinyint".into(),
            ColumnData::I16(_) => "smallint".into(),
            ColumnData::I32(_) => "int".into(),
            ColumnData::I64(_) => "bigint".into(),
            ColumnData::F32(_) => "float(24)".into(),
            ColumnData::F64(_) => "float(53)".into(),
            ColumnData::Bit(_) => "bit".into(),
            ColumnData::String(None) => "nvarchar(4000)".into(),
            ColumnData::String(Some(ref s)) if s.len() <= 4000 => "nvarchar(4000)".into(),
            ColumnData::String(Some(ref s)) if s.len() <= MAX_NVARCHAR_SIZE => {
                "nvarchar(max)".into()
            }
            ColumnData::String(_) => "ntext(max)".into(),
            ColumnData::Binary(Some(ref b)) if b.len() <= 8000 => "varbinary(8000)".into(),
            ColumnData::Binary(_) => "varbinary(max)".into(),
            ColumnData::Numeric(Some(ref n)) => {
                format!("numeric({},{})", n.precision(), n.scale()).into()
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

    pub fn decode(src: &mut BytesMut, ctx: &TypeInfo) -> Result<ColumnData<'a>> {
        todo!();
        // let res = match ctx {
        //     TypeInfo::FixedLen(fixed_ty) => fixed_len::decode(src, fixed_ty),
        //     TypeInfo::VarLenSized(cx) => var_len::decode(src, cx),
        //     TypeInfo::VarLenSizedPrecision { ty, scale, .. } => match ty {
        //         VarLenType::Decimaln | VarLenType::Numericn => {
        //             ColumnData::Numeric(Numeric::decode(src, *scale).await?)
        //         }
        //         _ => unreachable!("Unsupported type {:?}", ty),
        //     },
        // };

        // Ok(res)
    }

    pub fn encode(&self, dst: &mut BytesMut) -> Result<()> {
        match self {
            ColumnData::Bit(_)
            | ColumnData::U8(_)
            | ColumnData::I16(_)
            | ColumnData::I32(_)
            | ColumnData::I64(_)
            | ColumnData::F32(_)
            | ColumnData::F64(_) => fixed_len::encode(dst, &self)?,
            ColumnData::String(_) => string::encode(dst, &self)?,
            ColumnData::Date(_) => date::encode(dst, &self)?,
            ColumnData::DateTime2(_) => datetime2::encode(dst, &self)?,
            _ => unreachable!("ColumData of type {:?} is not supported", self),
        }

        Ok(())
    }
}
