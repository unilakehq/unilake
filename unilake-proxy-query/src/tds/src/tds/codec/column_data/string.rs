use crate::{Collation, ColumnData, Error, Result, VarLenType};
use byteorder::{ByteOrder, LittleEndian};
use encoding::DecoderTrap;
use std::borrow::Cow;
use tokio::io::{AsyncRead, AsyncWrite};

pub(crate) async fn decode<R>(
    src: &mut R,
    ty: VarLenType,
    len: usize,
    collation: Option<Collation>,
) -> Result<Option<Cow<'static, str>>>
where
    R: AsyncRead + Unpin,
{
    use VarLenType::*;

    let data = super::plp::decode(src, len).await?;

    match (data, ty) {
        // Codepages other than UTF
        (Some(buf), BigChar) | (Some(buf), BigVarChar) => {
            let collation = collation.as_ref().unwrap();
            let encoder = collation.encoding()?;

            let s: String = encoder
                .decode(buf.as_ref(), DecoderTrap::Strict)
                .map_err(Error::Encoding)?;

            Ok(Some(s.into()))
        }
        // UTF-16
        (Some(buf), _) => {
            if buf.len() % 2 != 0 {
                return Err(Error::Protocol("nvarchar: invalid plp length".into()));
            }

            let buf: Vec<_> = buf.chunks(2).map(LittleEndian::read_u16).collect();
            Ok(Some(String::from_utf16(&buf).unwrap().into()))
        }
        _ => Ok(None),
    }
}

pub(crate) async fn encode<W>(dst: &mut W, data: &ColumnData<'_>) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    //super::plp::encode(dst, data).await?;
    todo!();
    Ok(())
}
