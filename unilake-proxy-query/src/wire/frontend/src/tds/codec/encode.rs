use crate::Result;
use tokio_util::bytes::{BufMut, BytesMut};

pub fn write_us_varchar(dest: &mut BytesMut, s: &String) -> Result<()> {
    dest.put_u16_le(s.len() as u16);
    dest.put(s.as_bytes());
    Ok(())
}

pub fn write_b_varchar(dest: &mut BytesMut, s: &String) -> Result<()> {
    dest.put_u8(s.len() as u8);
    dest.put(s.as_bytes());
    Ok(())
}
