use crate::frontend::sqlstring::SqlString;
use crate::frontend::tds::codec::decode::read_us_varchar;
use crate::frontend::{ColumnData, VarLenContext, VarLenType};
use tokio_util::bytes::{BufMut, BytesMut};
use unilake_common::error::TdsWireResult;

/// Variable length token [2.2.4.2.1.3]
pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData) -> TdsWireResult<()> {
    match data {
        ColumnData::BitN(Some(val)) => {
            dst.put_u8(1);
            dst.put_u8(*val as u8);
        }
        ColumnData::U8N(Some(val)) => {
            dst.put_u8(1);
            dst.put_u8(*val);
        }
        ColumnData::I16N(Some(val)) => {
            dst.put_u8(2);
            dst.put_i16_le(*val);
        }
        ColumnData::I32N(Some(val)) => {
            dst.put_u8(4);
            dst.put_i32_le(*val);
        }
        ColumnData::I64N(Some(val)) => {
            dst.put_u8(8);
            dst.put_i64_le(*val);
        }
        ColumnData::F32N(Some(val)) => {
            dst.put_u8(4);
            dst.put_f32_le(*val);
        }
        ColumnData::F64N(Some(val)) => {
            dst.put_u8(8);
            dst.put_f64_le(*val);
        }
        _ => dst.put_u8(0),
    }

    // push null
    Ok(())
}

pub(crate) fn decode(
    src: &mut BytesMut,
    context: &VarLenContext,
) -> TdsWireResult<Option<ColumnData>> {
    // push null
    match context.r#type() {
        VarLenType::Intn => {}
        VarLenType::Bitn => {}
        VarLenType::Decimaln => {}
        VarLenType::Numericn => {}
        VarLenType::Floatn => {}
        VarLenType::Datetimen => {}
        VarLenType::Daten => {}
        VarLenType::Timen => {}
        VarLenType::Datetime2 => {}
        VarLenType::DatetimeOffsetn => {}
        VarLenType::BigVarBin => {}
        VarLenType::BigVarChar => {}
        VarLenType::BigBinary => {}
        VarLenType::BigChar => {}
        VarLenType::NVarchar => {
            let string = read_us_varchar(src)?;
            return Ok(Some(ColumnData::String(SqlString::from_string(
                string.into(),
                0,
            ))));
        }
        VarLenType::NChar => {}
        VarLenType::SSVariant => {}
    }

    Ok(None)
}
