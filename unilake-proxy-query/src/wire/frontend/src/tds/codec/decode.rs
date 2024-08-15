use crate::{
    error::{TdsWireError, TdsWireResult},
    Result,
};
use tokio_util::bytes::{Buf, BytesMut};

pub fn read_us_varchar(src: &mut BytesMut) -> Result<String> {
    let length = src.get_u16_le() as usize;
    read_string(src, length)
}

pub fn read_b_varchar(src: &mut BytesMut) -> TdsWireResult<String> {
    let length = src.get_u8() as usize;
    read_string(src, length)
}

fn read_string(src: &mut BytesMut, length: usize) -> TdsWireResult<String> {
    return if length > 0 {
        // Read the UTF-16 encoded bytes and decode them into a String
        let mut utf16_data = Vec::with_capacity(length * 2);
        (0..length).for_each(|_| utf16_data.push(src.get_u16_le()));

        // Convert the UTF-16 data String
        let result = String::from_utf16(&utf16_data)
            .map_err(|err| TdsWireError::Protocol(format!("Failed to decode varchar: {}", err)))?;

        Ok(result)
    } else {
        Ok(String::new())
    };
}
