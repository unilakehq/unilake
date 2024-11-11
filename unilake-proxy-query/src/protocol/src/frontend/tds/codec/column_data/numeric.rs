use crate::frontend::TdsTokenCodec;
use bigdecimal::BigDecimal;
use tokio_util::bytes::{BufMut, BytesMut};
use unilake_common::error::TdsWireResult;

pub(crate) fn encode(dest: &mut BytesMut, data: &Option<BigDecimal>) -> TdsWireResult<()> {
    if let Some(n) = data {
        n.encode(dest)?
    } else {
        // send null
        dest.put_u8(0);
    }
    Ok(())
}
