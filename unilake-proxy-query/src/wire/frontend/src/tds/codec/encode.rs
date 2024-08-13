use crate::Result;
use tokio_util::bytes::{BufMut, BytesMut};

pub fn write_us_varchar(dest: &mut BytesMut, s: &String) -> Result<()> {
    dest.put_u16_le(s.len() as u16);
    s.encode_utf16().for_each(|b| dest.put_u16_le(b));
    Ok(())
}

pub fn write_b_varchar(dest: &mut BytesMut, s: &String) -> Result<()> {
    dest.put_u8(s.len() as u8);
    s.encode_utf16().for_each(|b| dest.put_u16_le(b));
    Ok(())
}
