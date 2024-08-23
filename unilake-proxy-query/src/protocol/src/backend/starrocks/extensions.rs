use std::usize;

use enumflags2::BitFlags;
use mysql_async::Row;

use crate::frontend::{
    sqlstring::SqlString, BaseMetaDataColumn, ColumnData, MetaDataColumn, TokenRow, TypeInfo,
};

impl Into<MetaDataColumn> for &mysql_async::Column {
    fn into(self) -> MetaDataColumn {
        let name = String::from_utf8(self.name_ref().to_vec()).unwrap();
        let column_type = match self.column_type() {
            mysql_async::consts::ColumnType::MYSQL_TYPE_DECIMAL => TypeInfo::new_decimal(0, 0),
            // todo(mrhamburg): something is wrong with tinyint when encoding (client driver error)
            mysql_async::consts::ColumnType::MYSQL_TYPE_TINY => TypeInfo::new_tinyint(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_SHORT => TypeInfo::new_smallint(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_LONGLONG
            | mysql_async::consts::ColumnType::MYSQL_TYPE_LONG => TypeInfo::new_int(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_FLOAT => TypeInfo::new_float_32(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_DOUBLE => TypeInfo::new_float_64(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_NULL => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_TIMESTAMP => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_INT24 => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_DATE => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_TIME => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_DATETIME => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_YEAR => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_NEWDATE => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_VARCHAR => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_BIT => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_TIMESTAMP2 => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_DATETIME2 => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_TIME2 => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_TYPED_ARRAY => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_UNKNOWN => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_NEWDECIMAL => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_ENUM => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_SET => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_TINY_BLOB => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_MEDIUM_BLOB => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_LONG_BLOB => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_BLOB => todo!(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_JSON
            | mysql_async::consts::ColumnType::MYSQL_TYPE_STRING
            | mysql_async::consts::ColumnType::MYSQL_TYPE_VAR_STRING => {
                // TypeInfo::new_nvarchar(self.column_length() as usize)
                TypeInfo::new_string()
            }
            mysql_async::consts::ColumnType::MYSQL_TYPE_GEOMETRY => todo!(),
        };
        MetaDataColumn {
            base: BaseMetaDataColumn {
                flags: BitFlags::empty(),
                ty: column_type,
            },
            col_name: name,
        }
    }
}

impl Into<TokenRow> for Row {
    fn into(mut self) -> TokenRow {
        let mut row = TokenRow::new(self.columns_ref().len(), false);
        self.columns_ref()
            .to_vec()
            .iter()
            .enumerate()
            .map(|(i, c)| match c.column_type() {
                mysql_async::consts::ColumnType::MYSQL_TYPE_DECIMAL
                | mysql_async::consts::ColumnType::MYSQL_TYPE_NEWDECIMAL => {
                    // let x: Option<Decimal> = self.take(i);
                    // ColumnData::Numeric(x)
                    todo!()
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_TINY => {
                    let x: Option<u8> = self.take(i);
                    ColumnData::U8(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_SHORT => {
                    let x: Option<i16> = self.take(i);
                    ColumnData::I16(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_LONG
                | mysql_async::consts::ColumnType::MYSQL_TYPE_INT24 => {
                    let x: Option<i32> = self.take(i);
                    ColumnData::I32(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_LONGLONG => {
                    let x: Option<i64> = self.take(i);
                    ColumnData::I64(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_FLOAT => {
                    let x: Option<f32> = self.take(i);
                    ColumnData::F32(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_DOUBLE => {
                    let x: Option<f64> = self.take(i);
                    ColumnData::F64(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_BIT => {
                    let x: Option<bool> = self.take(i);
                    ColumnData::Bit(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_TIMESTAMP
                | mysql_async::consts::ColumnType::MYSQL_TYPE_TIMESTAMP2
                | mysql_async::consts::ColumnType::MYSQL_TYPE_DATETIME
                | mysql_async::consts::ColumnType::MYSQL_TYPE_DATETIME2 => {
                    // let x: Option<DateTime> = self.take(i);
                    // ColumnData::DateTime(x)
                    todo!()
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_TIME
                | mysql_async::consts::ColumnType::MYSQL_TYPE_TIME2 => {
                    // let x: Option<Time> = self.take(i);
                    // ColumnData::Time(x)
                    todo!()
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_DATE
                | mysql_async::consts::ColumnType::MYSQL_TYPE_NEWDATE => {
                    // let x: Option<Date> = self.take(i);
                    // ColumnData::Date(x)
                    todo!()
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_YEAR => {
                    let x: Option<i16> = self.take(i); // `YEAR` can be treated as an i16
                    ColumnData::I16(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_VARCHAR
                | mysql_async::consts::ColumnType::MYSQL_TYPE_VAR_STRING
                | mysql_async::consts::ColumnType::MYSQL_TYPE_STRING
                | mysql_async::consts::ColumnType::MYSQL_TYPE_JSON => {
                    let x: Option<String> = self.take_opt(i).unwrap().ok();
                    ColumnData::String(SqlString::from_string(x, usize::MAX))
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_ENUM
                | mysql_async::consts::ColumnType::MYSQL_TYPE_SET => {
                    // let x: Option<String> = self.take(i);
                    // ColumnData::String(SqlString::new(x.unwrap_or_default())) // Wrapping with SqlString
                    todo!()
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_TINY_BLOB
                | mysql_async::consts::ColumnType::MYSQL_TYPE_MEDIUM_BLOB
                | mysql_async::consts::ColumnType::MYSQL_TYPE_LONG_BLOB
                | mysql_async::consts::ColumnType::MYSQL_TYPE_BLOB => {
                    // let x: Option<Vec<u8>> = self.take(i);
                    // ColumnData::Binary(Some(base64::encode(x.unwrap_or_default())))
                    todo!()
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_TYPED_ARRAY => {
                    // Handle typed arrays as a string representation, assuming they are stored as JSON.
                    // let x: Option<String> = self.take(i);
                    // ColumnData::String(SqlString::new(x.unwrap_or_default()))
                    todo!()
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_UNKNOWN => {
                    // For unknown types, we'll return `None`
                    // ColumnData::String(SqlString::new("".to_string()))
                    todo!()
                }
                _ => ColumnData::Bit(Some(true)),
            })
            .for_each(|r| row.push_row(r));
        row
    }
}
