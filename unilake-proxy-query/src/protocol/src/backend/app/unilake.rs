use crate::backend::app::{FedResult, FedResultStream, FederatedRequestType, ResultSetBuilder};
use crate::frontend::{BatchRequest, ColumnData, DataFlags, TypeInfo};
use async_stream::stream;
use bigdecimal::BigDecimal;
use chrono::Utc;
use std::str::FromStr;
use unilake_common::error::TdsWireResult;

pub(crate) fn process_static(hash: u64, request: &FederatedRequestType) -> Option<FedResultStream> {
    let found = match request {
        FederatedRequestType::Query(query) => {
            match hash {
                // select internal_test_datatypes()
                3446327889591454507 => Some(test_datatypes(query)),
                // select internal_test_generic()
                17896409851857005090 => Some(test_generic(query)),
                _ => None,
            }
        }
        // todo: here we should also handle parsed queries like, show tables, create policy, etc..
        _ => None,
    };

    if let Some(result_set) = found {
        let stream = stream! {yield result_set;};
        return Some(FedResultStream::new(Box::pin(stream)));
    }

    None
}
fn test_generic(_: &BatchRequest) -> TdsWireResult<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column(
            Some("date"),
            TypeInfo::new_daten(false),
            DataFlags::default(),
        )
        .add_row(&[ColumnData::DateN(Some(Utc::now().naive_utc().date()))]);

    Ok(FedResult::Tabular(result_set.result))
}

fn test_datatypes(_: &BatchRequest) -> TdsWireResult<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column(
            Some("nvarchar(40)"),
            TypeInfo::new_nvarchar(Some(40)),
            DataFlags::default(),
        )
        .add_column(
            Some("big_int_nullable"),
            TypeInfo::new_big_intn(true),
            DataFlags::default(),
        )
        .add_column(
            Some("big_int_non_null"),
            TypeInfo::new_big_intn(false),
            DataFlags::default(),
        )
        .add_column(Some("bit"), TypeInfo::new_bit(), DataFlags::default())
        .add_column(
            Some("date"),
            TypeInfo::new_daten(false),
            DataFlags::default(),
        )
        .add_column(
            Some("datetime"),
            TypeInfo::new_datetime2(),
            DataFlags::default(),
        )
        .add_column(
            Some("decimal(19,2)"),
            TypeInfo::new_decimaln(19, 2),
            DataFlags::default(),
        )
        .add_column(
            Some("float_nullable"),
            TypeInfo::new_floatn_32(true),
            DataFlags::default(),
        )
        .add_column(
            Some("float_non_null"),
            TypeInfo::new_floatn_32(false),
            DataFlags::default(),
        )
        .add_column(
            Some("bigfloat_nullable"),
            TypeInfo::new_floatn_64(true),
            DataFlags::default(),
        )
        .add_column(
            Some("bigfloat_non_null"),
            TypeInfo::new_floatn_64(false),
            DataFlags::default(),
        )
        .add_column(
            Some("int_nullable"),
            TypeInfo::new_intn(true),
            DataFlags::default(),
        )
        .add_column(
            Some("int_non_null"),
            TypeInfo::new_intn(false),
            DataFlags::default(),
        )
        .add_column(
            Some("smallint_nullable"),
            TypeInfo::new_small_intn(true),
            DataFlags::default(),
        )
        .add_column(
            Some("smallint_non_null"),
            TypeInfo::new_small_intn(false),
            DataFlags::default(),
        )
        .add_column(
            Some("tinyint_nullable"),
            TypeInfo::new_tiny_intn(true),
            DataFlags::default(),
        )
        .add_column(
            Some("tinyint_non_null"),
            TypeInfo::new_tiny_intn(false),
            DataFlags::default(),
        )
        .add_row(&[
            ColumnData::new_nvarchar(Some("Unilake SQL Proxy".to_owned()), Some(40)),
            ColumnData::I64N(None),
            ColumnData::I64(1988),
            ColumnData::Bit(true),
            ColumnData::DateN(Some(Utc::now().naive_utc().date())),
            ColumnData::DateTime2(Some(Utc::now().naive_utc())),
            ColumnData::Numeric(Some(BigDecimal::from_str("999999999999999.99").unwrap())),
            ColumnData::F32N(None),
            ColumnData::F32(123.123),
            ColumnData::F64N(None),
            ColumnData::F64(123.45),
            ColumnData::I32N(None),
            ColumnData::I32(123111),
            ColumnData::I16N(None),
            ColumnData::I16(123),
            ColumnData::U8N(None),
            ColumnData::U8(12),
        ]);

    Ok(FedResult::Tabular(result_set.result))
}
