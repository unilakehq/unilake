use crate::{utils::ReadAndAdvance, Result};
use tokio_util::bytes::{Buf, BytesMut};

pub fn read_us_varchar(src: &mut BytesMut) -> Result<String> {
    let lines = src.get_u16_le() as usize * 2;
    return if lines > 0 {
        let (count, chars) = src.read_and_advance(lines);
        if count != lines {
            // todo(mrhamburg): Handle partial read
            panic!("Expected {} bytes, but only {} were read", lines, count);
        }
        Ok(String::from_utf8(chars).unwrap())
    } else {
        Ok(String::new())
    };
}

pub fn read_b_varchar(src: &mut BytesMut) -> Result<String> {
    let lines = src.get_u8() as usize * 2;
    return if lines > 0 {
        let (count, chars) = src.read_and_advance(lines);
        if count != lines {
            // todo(mrhamburg): Handle partial read
            panic!("Expected {} bytes, but only {} were read", lines, count);
        }
        let result = String::from_utf8(chars);
        Ok(result.unwrap())
    } else {
        Ok(String::new())
    };
}
