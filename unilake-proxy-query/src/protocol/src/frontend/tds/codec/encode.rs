use tokio_util::bytes::{BufMut, BytesMut};
use unilake_common::error::TdsWireResult;

pub fn write_us_varchar(dest: &mut BytesMut, s: &String) -> TdsWireResult<()> {
    let length = s.len();

    // US_VARCHAR stores the length as 2 bytes, so the length must not exceed 65535
    if length > 65535 {
        return Err(unilake_common::error::Error::Protocol(
            "String length exceeds maximum US_VARCHAR size of 65535".to_string(),
        ));
    }
    dest.put_u16_le(length as u16);
    s.encode_utf16().for_each(|b| dest.put_u16_le(b));

    Ok(())
}

pub fn write_b_varchar(dest: &mut BytesMut, s: &String) -> TdsWireResult<()> {
    let length = s.len();

    // B_VARCHAR stores the length as a single byte, so the length must not exceed 255
    if length > 255 {
        return Err(unilake_common::error::Error::Protocol(
            "String length exceeds maximum B_VARCHAR size of 255".to_string(),
        ));
    }
    dest.put_u8(length as u8);
    s.encode_utf16().for_each(|b| dest.put_u16_le(b));

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::frontend::tds::codec::{decode, encode};
    use tokio_util::bytes::BytesMut;
    use unilake_common::error::TdsWireResult;

    const RAW_BYTES_B_VARCHAR: &[u8] = &[
        0x14, 0x4d, 0x00, 0x69, 0x00, 0x63, 0x00, 0x72, 0x00, 0x6f, 0x00, 0x73, 0x00, 0x6f, 0x00,
        0x66, 0x00, 0x74, 0x00, 0x20, 0x00, 0x53, 0x00, 0x51, 0x00, 0x4c, 0x00, 0x20, 0x00, 0x53,
        0x00, 0x65, 0x00, 0x72, 0x00, 0x76, 0x00, 0x65, 0x00, 0x72, 0x00,
    ];

    #[test]
    fn encode_decode_roundtrip_us_varchar() {}

    #[test]
    fn encode_decode_roundtrip_b_varchar() -> TdsWireResult<()> {
        let mut buff = BytesMut::from(&RAW_BYTES_B_VARCHAR[..]);
        let decoded = decode::read_b_varchar(&mut buff)?;
        encode::write_b_varchar(&mut buff, &decoded)?;

        assert_eq!(buff.to_vec(), RAW_BYTES_B_VARCHAR.to_vec());

        Ok(())
    }
}
