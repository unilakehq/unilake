// todo(mrhamburg): determine if this is really needed and y'all
use crate::tds::codec::ColumnData;
use std::borrow::Cow;

/// A conversion trait to a TDS type.
///
/// A `ToSql` implementation for a Rust type, the
/// following Rust types are already implemented to match the given server
/// types:
///
/// |Rust type|Server type|
/// |--------|--------|
/// |`u8`|`tinyint`|
/// |`i16`|`smallint`|
/// |`i32`|`int`|
/// |`i64`|`bigint`|
/// |`f32`|`float(24)`|
/// |`f64`|`float(53)`|
/// |`bool`|`bit`|
/// |`String`/`&str`|`nvarchar(max)`|
/// |`NaiveDate`|`date`|
/// |`NaiveDateTime`|`datetime2`|
/// TODO: ssvariant
pub trait ToSql: Send + Sync {
    /// Convert to a value understood by the SQL Server. Conversion
    /// by-reference.
    fn to_sql(&self) -> ColumnData;
}

/// A by-value conversion trait to a TDS type.
pub trait IntoSql<'a>: Send + Sync {
    /// Convert to a value understood by the SQL Server. Conversion by-value.
    fn into_sql(self) -> ColumnData;
}

impl<'a> IntoSql<'a> for &'a str {
    fn into_sql(self) -> ColumnData {
        ColumnData::String(Some(self.to_string()))
    }
}

// impl<'a> IntoSql<'a> for Option<&'a str> {
//     fn into_sql(self) -> ColumnData {
//         ColumnData::String(self.map(Cow::Borrowed))
//     }
// }

// impl<'a> IntoSql<'a> for &'a String {
//     fn into_sql(self) -> ColumnData<'a> {
//         ColumnData::String(Some(Cow::Borrowed(self)))
//     }
// }

// impl<'a> IntoSql<'a> for Option<&'a String> {
//     fn into_sql(self) -> ColumnData<'a> {
//         ColumnData::String(self.map(Cow::from))
//     }
// }

// impl<'a> IntoSql<'a> for &'a [u8] {
//     fn into_sql(self) -> ColumnData<'a> {
//         ColumnData::Binary(Some(Cow::Borrowed(self)))
//     }
// }

// impl<'a> IntoSql<'a> for Option<&'a [u8]> {
//     fn into_sql(self) -> ColumnData<'a> {
//         ColumnData::Binary(self.map(Cow::Borrowed))
//     }
// }

// impl<'a> IntoSql<'a> for &'a Vec<u8> {
//     fn into_sql(self) -> ColumnData<'a> {
//         ColumnData::Binary(Some(Cow::from(self)))
//     }
// }

// impl<'a> IntoSql<'a> for Option<&'a Vec<u8>> {
//     fn into_sql(self) -> ColumnData<'a> {
//         ColumnData::Binary(self.map(Cow::from))
//     }
// }

// impl<'a> IntoSql<'a> for Cow<'a, str> {
//     fn into_sql(self) -> ColumnData<'a> {
//         ColumnData::String(Some(self))
//     }
// }

// impl<'a> IntoSql<'a> for Option<Cow<'a, str>> {
//     fn into_sql(self) -> ColumnData<'a> {
//         ColumnData::String(self)
//     }
// }

// impl<'a> IntoSql<'a> for Cow<'a, [u8]> {
//     fn into_sql(self) -> ColumnData<'a> {
//         ColumnData::Binary(Some(self))
//     }
// }

// impl<'a> IntoSql<'a> for Option<Cow<'a, [u8]>> {
//     fn into_sql(self) -> ColumnData<'a> {
//         ColumnData::Binary(self)
//     }
// }

// TODO: fix this
// into_sql!(self_,
//           String: (ColumnData::String, Cow::from(self_));
//           Vec<u8>: (ColumnData::Binary, Cow::from(self_));
//           bool: (ColumnData::Bit, self_);
//           u8: (ColumnData::U8, self_);
//           i16: (ColumnData::I16, self_);
//           i32: (ColumnData::I32, self_);
//           i64: (ColumnData::I64, self_);
//           f32: (ColumnData::F32, self_);
//           f64: (ColumnData::F64, self_);
// );
//
// to_sql!(self_,
//         bool: (ColumnData::Bit, *self_);
//         u8: (ColumnData::U8, *self_);
//         i16: (ColumnData::I16, *self_);
//         i32: (ColumnData::I32, *self_);
//         i64: (ColumnData::I64, *self_);
//         f32: (ColumnData::F32, *self_);
//         f64: (ColumnData::F64, *self_);
//         &str: (ColumnData::String, Cow::from(*self_));
//         String: (ColumnData::String, Cow::from(self_));
//         Cow<'_, str>: (ColumnData::String, self_.clone());
//         &[u8]: (ColumnData::Binary, Cow::from(*self_));
//         Cow<'_, [u8]>: (ColumnData::Binary, self_.clone());
//         Vec<u8>: (ColumnData::Binary, Cow::from(self_));
// );
