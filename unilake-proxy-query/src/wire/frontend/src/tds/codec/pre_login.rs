use crate::tds::codec::guid::reorder_bytes;
use crate::utils::ReadAndAdvance;
use crate::{tds::EncryptionLevel, Error, Result};
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use uuid::Uuid;

/// Client application activity id token used for debugging purposes introduced in TDS 7.4.
#[allow(unused)]
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ActivityId {
    id: Uuid,
    sequence: u32,
}

/// The prelogin packet used to initialize a connection [2.2.6.5]
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct PreloginMessage {
    /// [BE] token=0x00
    /// Either the driver version or the version of the SQL server
    pub version: u32,
    pub sub_build: u16,
    /// token=0x01
    pub encryption: Option<EncryptionLevel>,
    /// token=0x02
    pub instance_name: Option<String>,
    /// [client] threadid for debugging purposes, token=0x03
    pub thread_id: u32,
    /// token=0x04
    pub mars: bool,
    /// token=0x05
    pub activity_id: Option<ActivityId>,
    /// token=0x06
    pub fed_auth_required: Option<bool>,
    pub nonce: Option<[u8; 32]>,
}

// prelogin fields
// http://msdn.microsoft.com/en-us/library/dd357559.aspx
const PRELOGIN_VERSION: u8 = 0;
const PRELOGIN_ENCRYPTION: u8 = 1;
const PRELOGIN_INSTOPT: u8 = 2;
const PRELOGIN_THREADID: u8 = 3;
const PRELOGIN_MARS: u8 = 4;
const PRELOGIN_TRACEID: u8 = 5;
const PRELOGIN_FEDAUTHREQUIRED: u8 = 6;
const PRELOGIN_NONCEOPT: u8 = 7;
const PRELOGIN_TERMINATOR: u8 = 0xff;

impl PreloginMessage {
    pub fn new() -> PreloginMessage {
        let driver_version = crate::get_driver_version();
        PreloginMessage {
            version: driver_version as u32,
            sub_build: (driver_version >> 32) as u16,
            encryption: Some(EncryptionLevel::NotSupported),
            instance_name: None,
            thread_id: 0,
            mars: false,
            activity_id: None,
            fed_auth_required: Some(false),
            nonce: None,
        }
    }

    pub fn encode(&self, dst: &mut BytesMut) -> Result<()> {
        // create headers
        let mut options = Vec::<(u8, u16, u16)>::with_capacity(3);
        options.push((PRELOGIN_VERSION, 6, 0));
        options.push((PRELOGIN_THREADID, 4, 0));
        options.push((PRELOGIN_MARS, 1, 0));

        if self.activity_id.is_some() {
            options.push((PRELOGIN_TRACEID, 36, 0));
        }
        if self.fed_auth_required.is_some() & self.fed_auth_required.unwrap() {
            options.push((PRELOGIN_FEDAUTHREQUIRED, 1, 0));
        }
        if self.nonce.is_some() {
            options.push((PRELOGIN_NONCEOPT, 32, 0));
        }
        options.push((PRELOGIN_ENCRYPTION, 1, 0));
        options.push((PRELOGIN_TERMINATOR, 0, 0));

        // get current offset (5 bytes for each option, except for the terminator, which is 1 byte)
        let mut current_offset: u16 = (options.len() * 5 - 4) as u16;
        for i in 0..options.len() {
            options[i].2 = current_offset;
            current_offset += options[i].1;
        }

        // write token headers
        for i in 0..options.len() {
            let option = &options[i];
            // type
            dst.put_u8(option.0);
            if option.0 != PRELOGIN_TERMINATOR {
                // position
                dst.put_u16(option.2);
                // length
                dst.put_u16(option.1);
            }
        }

        // write version
        dst.put_u32(self.version);
        dst.put_u16(self.sub_build);

        // write thread_id
        dst.put_u32(self.thread_id);

        // write mars
        dst.put_u8(self.mars as u8);

        // TODO: I believe we can skip this
        // write trace_id
        // if self.activity_id.is_some(){
        //     dst.write_u32(self.activity_id.unwrap()).await?;
        // }

        // write fed_auth_required
        if self.fed_auth_required.is_some() & self.fed_auth_required.unwrap() {
            dst.put_u8(self.fed_auth_required.unwrap() as u8);
        }

        // write nonce
        if self.nonce.is_some() {
            dst.put(self.nonce.unwrap().as_slice());
        }

        // write encryption
        if self.encryption.is_some() {
            dst.put_u8(self.encryption.unwrap() as u8);
        }

        Ok(())
    }

    pub fn decode(src: &mut BytesMut) -> Result<Self> {
        let mut ret = PreloginMessage::new();
        let options = {
            let mut options = Vec::new();
            loop {
                let token = src.get_u8();

                // read until terminator
                if token == 0xff {
                    break;
                }
                let position = src.get_u16();
                let length = src.get_u16();
                options.push((token, position, length));
            }

            options.sort_by(|a, b| a.1.cmp(&b.1));
            options
        };

        // get initial offset
        let mut decode_offset_initial: u16 = options.len() as u16 * 5u16 + 1u16;
        // read all options
        for option in options.iter().enumerate() {
            let token = option.1 .0;
            let position = option.1 .1;
            let length = option.1 .2;

            while decode_offset_initial < position {
                let _ = src.get_u8();
                decode_offset_initial += 1;
            }

            // verify whether the server acts in accordance to what we requested
            // and if we can handle on what we seemingly agreed to
            match token {
                // version
                PRELOGIN_VERSION => {
                    ret.version = src.get_u32();
                    ret.sub_build = src.get_u16();
                    decode_offset_initial += 6;
                }
                // encryption
                PRELOGIN_ENCRYPTION => {
                    let encrypt = src.get_u8();
                    ret.encryption =
                        Some(crate::tds::EncryptionLevel::try_from(encrypt).map_err(|_| {
                            Error::Protocol(format!("invalid encryption value: {}", encrypt).into())
                        })?);
                    decode_offset_initial += 1;
                }
                // instance name
                PRELOGIN_INSTOPT => {
                    let mut bytes = Vec::new();
                    let mut next_byte = src.get_u8();
                    decode_offset_initial += 1;

                    while next_byte != 0x00 {
                        bytes.push(next_byte);
                        next_byte = src.get_u8();
                        decode_offset_initial += 1;
                    }

                    if !bytes.is_empty() {
                        ret.instance_name = Some(String::from_utf8_lossy(&bytes).into_owned());
                    }
                }
                PRELOGIN_THREADID => {
                    ret.thread_id = if length == 0 {
                        0
                    } else if length == 4 {
                        src.get_u32()
                    } else {
                        panic!("should never happen")
                    };
                    decode_offset_initial += 4;
                }
                // mars
                PRELOGIN_MARS => {
                    ret.mars = src.get_u8() == 0x01;
                    decode_offset_initial += 1;
                }
                // activity id
                PRELOGIN_TRACEID => {
                    // Data is a Guid, 16 bytes and ordered the wrong way around than Uuid.
                    let mut data = [0u8; 16];
                    src.get(0..data.len());
                    src.advance(data.len());
                    reorder_bytes(&mut data);

                    ret.activity_id = Some(ActivityId {
                        id: Uuid::from_bytes(data),
                        sequence: src.get_u32_le(),
                    });
                    decode_offset_initial += 36;
                }
                // fed auth
                PRELOGIN_FEDAUTHREQUIRED => {
                    ret.fed_auth_required = Some(src.get_u8() != 0);
                    decode_offset_initial += 1;
                }
                // nonce
                PRELOGIN_NONCEOPT => {
                    let mut data = [0u8; 32];
                    src.get(0..data.len());
                    src.advance(data.len());
                    ret.nonce = Some(data);
                    decode_offset_initial += 32;
                }
                _ => panic!("unsupported pre-login token: {}", token),
            }
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelogin_roundtrip() -> Result<()> {
        let input = PreloginMessage::new();

        // arrange
        let mut src = BytesMut::new();

        // encode
        input.encode(&mut src).unwrap();

        // decode
        let result = PreloginMessage::decode(&mut src).unwrap();

        // assert
        assert_eq!(input.version, result.version);
        assert_eq!(input.sub_build, result.sub_build);
        assert_eq!(input.mars, result.mars);
        assert_eq!(input.thread_id, result.thread_id);
        assert_eq!(input.fed_auth_required, result.fed_auth_required);
        assert_eq!(input.encryption, result.encryption);

        Ok(())
    }

    #[test]
    fn prelogin_with_fedauth_roundtrip() {
        todo!("");
    }
}
