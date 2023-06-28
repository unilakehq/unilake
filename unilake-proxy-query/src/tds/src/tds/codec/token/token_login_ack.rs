use crate::tds::codec::{decode, encode};
use crate::{Error, FeatureLevel, Result, TokenType};
use std::convert::TryFrom;
use std::mem::size_of;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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
    pub(crate) async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let _length = src.read_u16_le().await?;

        let interface = src.read_u8().await?;

        let tds_version = FeatureLevel::try_from(src.read_u32().await?)
            .map_err(|_| Error::Protocol("Login ACK: Invalid TDS version".into()))?;

        let prog_name = decode::read_b_varchar(src).await?;
        let version = src.read_u32_le().await?;

        Ok(TokenLoginAck {
            interface,
            tds_version,
            prog_name,
            version,
        })
    }
    pub(crate) async fn encode<W>(&mut self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TokenType::LoginAck as u8).await?;
        let prog_name_len = self.prog_name.len();
        let len = (
            size_of::<u8>() // interface
                + size_of::<u32>() // protocol version
                + size_of::<u8>() + prog_name_len // prog name
                + size_of::<u32>()
            // version
        ) as u16;

        dest.write_u16_le(len).await?;
        dest.write_u8(self.interface).await?;
        dest.write_u32(self.tds_version as u32).await?;
        encode::write_b_varchar(dest, &self.prog_name).await?;
        dest.write_u32_le(self.version).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{FeatureLevel, Result, TokenLoginAck, TokenType};
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

    #[tokio::test]
    async fn encode_decode_token_login_ack() -> Result<()> {
        let mut input = TokenLoginAck {
            interface: 12,
            prog_name: "test".to_string(),
            version: 0x74000004,
            tds_version: FeatureLevel::SqlServerN,
        };

        // arrange
        let (inner, outer) = tokio::io::duplex(256);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input.encode(&mut writer).await?;
        writer.flush().await?;

        // decode
        let tokentype = reader.read_u8().await?;
        let result = TokenLoginAck::decode(&mut reader).await?;

        // assert
        assert_eq!(tokentype, TokenType::LoginAck as u8);
        assert_eq!(result.interface, input.interface);
        assert_eq!(result.prog_name, input.prog_name);
        assert_eq!(result.version, input.version);
        assert_eq!(result.tds_version, input.tds_version);

        Ok(())
    }
}
