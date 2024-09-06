use crate::frontend::Result;
use crate::frontend::TdsTokenCodec;
use bigdecimal::BigDecimal;
use tokio_util::bytes::{BufMut, BytesMut};

pub(crate) fn encode(dest: &mut BytesMut, data: &Option<BigDecimal>) -> Result<()> {
    if let Some(n) = data {
        n.encode(dest)?
    } else {
        // send null
        dest.put_u8(0);
    }
    Ok(())
}
