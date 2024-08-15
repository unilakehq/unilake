use crate::tds::codec::{decode, encode};
use crate::tds::server_context::ServerContext;
use crate::{Error, FeatureLevel, Result, TdsToken, TdsTokenCodec, TdsTokenType};
use std::convert::TryFrom;
use std::sync::Arc;
use tokio_util::bytes::{Buf, BufMut, BytesMut};

/// LoginAck Token [2.2.7.14]
/// Used to send a response to a login request (LOGIN7) to the client.
#[allow(dead_code)] // we might want to debug the values
#[derive(Debug)]
pub struct TokenLoginAck {
    /// The type of interface with which the server will accept client requests
    /// 0: SQL_DFLT (server confirms that whatever is sent by the client is acceptable. If the client
    ///    requested SQL_DFLT, SQL_TSQL will be used)
    /// 1: SQL_TSQL (TSQL is accepted)
    pub interface: u8,
    pub tds_version: FeatureLevel,
    pub prog_name: String,
    /// major.minor.buildhigh.buildlow
    pub version: u32,
}

impl TokenLoginAck {
    pub fn new(server_contex: Arc<ServerContext>) -> Self {
        TokenLoginAck {
            interface: 1, // SQL_TSQL
            tds_version: FeatureLevel::SqlServerN,
            prog_name: server_contex.server_name.clone(),
            version: server_contex.get_server_version(),
        }
    }
}

impl TdsTokenCodec for TokenLoginAck {
    fn decode(src: &mut BytesMut) -> Result<TdsToken> {
        let _length = src.get_u16_le();

        let interface = src.get_u8();

        let tds_version = FeatureLevel::try_from(src.get_u32())
            .map_err(|_| Error::Protocol("Login ACK: Invalid TDS version".to_string()))?;

        let prog_name = decode::read_b_varchar(src)?;
        let version = src.get_u32_le();

        Ok(TdsToken::LoginAck(TokenLoginAck {
            interface,
            tds_version,
            prog_name,
            version,
        }))
    }
    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::LoginAck as u8);
        let mut buff = BytesMut::new();

        // set content
        buff.put_u8(self.interface);
        buff.put_u32(self.tds_version as u32);
        encode::write_b_varchar(&mut buff, &self.prog_name)?;
        buff.put_u32_le(self.version);

        // push length and content
        dest.put_u16_le(buff.len() as u16);
        dest.extend_from_slice(&buff);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::TdsWireResult, FeatureLevel, Result, TdsToken, TdsTokenCodec, TdsTokenType,
        TokenLoginAck,
    };
    use tokio_util::bytes::{Buf, BytesMut};

    const RAW_BYTES: &[u8] = &[
        0xAD, 0x36, 0x00, 0x01, 0x74, 0x00, 0x00, 0x04, 0x16, 0x4d, 0x00, 0x69, 0x00, 0x63, 0x00,
        0x72, 0x00, 0x6f, 0x00, 0x73, 0x00, 0x6f, 0x00, 0x66, 0x00, 0x74, 0x00, 0x20, 0x00, 0x53,
        0x00, 0x51, 0x00, 0x4c, 0x00, 0x20, 0x00, 0x53, 0x00, 0x65, 0x00, 0x72, 0x00, 0x76, 0x00,
        0x65, 0x00, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x10, 0x27,
    ];

    #[test]
    fn decode_encode_raw() -> TdsWireResult<()> {
        let mut bytes = BytesMut::from(&RAW_BYTES[1..]);
        let messsage = TokenLoginAck::decode(&mut bytes)?;

        let mut buff = BytesMut::new();
        if let TdsToken::LoginAck(messsage) = messsage {
            messsage.encode(&mut buff)?;
        }
        assert_eq!(RAW_BYTES, buff.to_vec());

        Ok(())
    }

    #[test]
    fn encode_decode_token_login_ack() -> Result<()> {
        let input = TokenLoginAck {
            interface: 12,
            prog_name: "test".to_string(),
            version: 0x74000004,
            tds_version: FeatureLevel::SqlServerN,
        };

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).expect("should be ok");

        // decode
        let tokentype = buff.get_u8();
        let result = TokenLoginAck::decode(&mut buff).unwrap();

        // assert
        assert_eq!(tokentype, TdsTokenType::LoginAck as u8);
        if let TdsToken::LoginAck(result) = result {
            assert_eq!(result.interface, input.interface);
            assert_eq!(result.prog_name, input.prog_name);
            assert_eq!(result.version, input.version);
            assert_eq!(result.tds_version, input.tds_version);
        } else {
            panic!("Expected token LoginAck")
        }
        Ok(())
    }
}
