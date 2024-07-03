use crate::{ColumnData, Result, VarLenContext, VarLenType};
use tokio::io::{AsyncRead, AsyncReadExt};

/// Var length token [2.2.4.2.1.3]
pub(crate) async fn decode<R>(src: &mut R, ctx: &VarLenContext) -> Result<ColumnData<'static>>
where
    R: AsyncRead + Unpin,
{
    use VarLenType::*;

    let ty = ctx.r#type();
    let len = ctx.len();
    let collation = ctx.collation();

    let res = match ty {
        Bitn => super::bit::decode(src).await?,
        Intn => super::int::decode(src, len).await?,
        Floatn => super::float::decode(src, len).await?,
        BigChar | BigVarChar | NChar | NVarchar => {
            ColumnData::String(super::string::decode(src, ty, len, collation).await?)
        }
        Datetimen => {
            let len = src.read_u8().await?;
            super::datetimen::decode(src, len).await?
        }
        Daten => super::date::decode(src).await?,
        Timen => super::time::decode(src, len).await?,
        Datetime2 => super::datetime2::decode(src, len as usize).await?,
        DatetimeOffsetn => super::datetimeoffsetn::decode(src, len as usize).await?,
        BigBinary | BigVarBin => super::binary::decode(src, len).await?,
        t => unimplemented!("{:?}", t),
    };

    Ok(res)
}
