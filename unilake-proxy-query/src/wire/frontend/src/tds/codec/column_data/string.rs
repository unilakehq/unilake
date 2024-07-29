use crate::{Collation, ColumnData, Error, Result, VarLenType};
use byteorder::{ByteOrder, LittleEndian};
use encoding::DecoderTrap;
use std::borrow::Cow;
use tokio_util::bytes::BytesMut;

pub(crate) fn decode(
    src: &mut BytesMut,
    ty: VarLenType,
    len: usize,
    collation: Option<Collation>,
) -> Result<Option<Cow<'static, str>>> {
    use VarLenType::*;

    let data = super::plp::decode(src, len)?;

    match (data, ty) {
        // Codepages other than UTF
        (Some(buf), BigChar) | (Some(buf), BigVarChar) => {
            let collation = collation.as_ref().unwrap();
            let encoder = collation.encoding()?;

            todo!("fix this")
            // let s: String = encoder
            //     .decode(buf.as_ref(), DecoderTrap::Strict)
            //     .map_err(Error::Encoding)?;

            // Ok(Some(s.into()))
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

pub(crate) fn encode(dst: &mut BytesMut, data: &ColumnData<'_>) -> Result<()> {
    //super::plp::encode(dst, data).await?;
    todo!();
    Ok(())
}
