use crate::Result;
use tokio_util::bytes::{Buf, BytesMut};

pub fn read_us_varchar(src: &mut BytesMut) -> Result<String> {
    let lines = src.get_u16_le();
    return if lines > 0 {
        let mut chars = Vec::with_capacity(lines as usize);
        for _ in 0..lines {
            chars.push(src.get_u8());
        }
        Ok(String::from_utf8(chars).unwrap())
    } else {
        Ok(String::new())
    };
}

pub fn read_b_varchar(src: &mut BytesMut) -> Result<String> {
    let lines = src.get_u8();
    return if lines > 0 {
        let mut chars = Vec::with_capacity(lines as usize);
        for _ in 0..lines {
            chars.push(src.get_u8());
        }
        Ok(String::from_utf8(chars).unwrap())
    } else {
        Ok(String::new())
    };
}
