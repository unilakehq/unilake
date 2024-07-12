use crate::{ColumnData, Result, VarLenContext, VarLenType};
use tokio_util::bytes::{Buf, BytesMut};

/// Var length token [2.2.4.2.1.3]
pub(crate) fn decode(src: &mut BytesMut, ctx: &VarLenContext) -> Result<ColumnData<'static>> {
    use VarLenType::*;

    let ty = ctx.r#type();
    let len = ctx.len();
    // let collation = ctx.collation();

    let res = match ty {
        Bitn => super::bit::decode(src),
        Intn => super::int::decode(src, len),
        Floatn => super::float::decode(src, len),
        BigChar | BigVarChar | NChar | NVarchar => {
            todo!()
            // ColumnData::String(super::string::decode(src, ty, len, collation)?)
        }
        Datetimen => {
            let len = src.get_u8();
            super::datetimen::decode(src, len)
        }
        Daten => super::date::decode(src),
        Timen => super::time::decode(src, len),
        Datetime2 => super::datetime2::decode(src, len as usize),
        DatetimeOffsetn => super::datetimeoffsetn::decode(src, len as usize),
        BigBinary | BigVarBin => super::binary::decode(src, len),
        t => unimplemented!("{:?}", t),
    };

    Ok(res?)
}
