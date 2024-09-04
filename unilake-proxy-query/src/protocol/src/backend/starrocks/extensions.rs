use std::usize;

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use enumflags2::BitFlags;
use mysql_async::{prelude::FromValue, Row};

use crate::frontend::{
    sqlstring::SqlString, BaseMetaDataColumn, ColumnData, DateTime2, MetaDataColumn, TokenRow,
    TypeInfo,
};

impl Into<MetaDataColumn> for &mysql_async::Column {
    fn into(self) -> MetaDataColumn {
        let name = String::from_utf8(self.name_ref().to_vec()).unwrap();
        let column_type = match self.column_type() {
            mysql_async::consts::ColumnType::MYSQL_TYPE_NEWDECIMAL
            | mysql_async::consts::ColumnType::MYSQL_TYPE_DECIMAL => {
                TypeInfo::new_decimal(self.column_length() as u8, self.decimals())
            }
            mysql_async::consts::ColumnType::MYSQL_TYPE_TINY => TypeInfo::new_tinyint(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_SHORT => TypeInfo::new_smallint(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_LONGLONG => TypeInfo::new_bigint(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_LONG => TypeInfo::new_int(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_FLOAT => TypeInfo::new_float_32(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_DOUBLE => TypeInfo::new_float_64(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_JSON
            | mysql_async::consts::ColumnType::MYSQL_TYPE_BLOB
            | mysql_async::consts::ColumnType::MYSQL_TYPE_STRING
            | mysql_async::consts::ColumnType::MYSQL_TYPE_VAR_STRING => {
                // TypeInfo::new_nvarchar(self.column_length() as usize)
                // if self.column_length() < 240 {
                //     TypeInfo::new_nvarchar(self.column_length() as usize)
                // } else {
                //     TypeInfo::new_string()
                // }
                TypeInfo::new_string()
            }
            mysql_async::consts::ColumnType::MYSQL_TYPE_DATETIME => TypeInfo::new_datetime(),
            mysql_async::consts::ColumnType::MYSQL_TYPE_DATE => TypeInfo::new_date(),
            _ => {
                tracing::error!("Unknown column type: {:?}", self.column_type());
                unreachable!()
            }
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
                    let x: Option<BigDecimal> = self.take(i);
                    ColumnData::Numeric(x)
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
                    let x: Option<NaiveDateTime> = self.take(i);
                    ColumnData::DateTime2(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_TIME
                | mysql_async::consts::ColumnType::MYSQL_TYPE_TIME2 => {
                    let x: Option<NaiveTime> = self.take(i);
                    ColumnData::Time(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_DATE
                | mysql_async::consts::ColumnType::MYSQL_TYPE_NEWDATE => {
                    let x: Option<NaiveDate> = self.take(i);
                    ColumnData::Date(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_YEAR => {
                    let x: Option<i16> = self.take(i); // `YEAR` can be treated as an i16
                    ColumnData::I16(x)
                }
                mysql_async::consts::ColumnType::MYSQL_TYPE_VARCHAR
                | mysql_async::consts::ColumnType::MYSQL_TYPE_TINY_BLOB
                | mysql_async::consts::ColumnType::MYSQL_TYPE_MEDIUM_BLOB
                | mysql_async::consts::ColumnType::MYSQL_TYPE_LONG_BLOB
                | mysql_async::consts::ColumnType::MYSQL_TYPE_BLOB
                | mysql_async::consts::ColumnType::MYSQL_TYPE_VAR_STRING
                | mysql_async::consts::ColumnType::MYSQL_TYPE_STRING
                | mysql_async::consts::ColumnType::MYSQL_TYPE_JSON => {
                    // todo(mrhamburg): lets not do an unwrap here
                    let x: Option<String> = self.take_opt(i).unwrap().ok();
                    ColumnData::String(SqlString::from_string(x, usize::MAX))
                }
                _ => unimplemented!(),
            })
            .for_each(|r| row.push_row(r));
        row
    }
}
