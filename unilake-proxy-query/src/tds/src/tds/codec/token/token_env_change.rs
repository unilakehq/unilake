//TODO: fix unwraps in this file for proper error handling
use crate::tds::codec::encode;
use crate::{Error, Result, TokenType};
use std::fmt::Debug;
use std::io::Cursor;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};

uint_enum! {
    /// Environment change token type [2.2.7.9]
    /// The type of environment change
    #[repr(u8)]
    pub enum EnvChangeType {
        Database = 1,
        Language = 2,
        CharacterSet = 3,
        PacketSize = 4,
        UnicodeDataSortingLID = 5,
        UnicodeDataSortingCFL = 6,
        SqlCollation = 7,
        /// below here: >= TDSv7.2
        BeginTransaction = 8,
        CommitTransaction = 9,
        RollbackTransaction = 10,
        EnlistDTCTransaction = 11,
        DefectTransaction = 12,
        Rtls = 13,
        PromoteTransaction = 15,
        TransactionManagerAddress = 16,
        TransactionEnded = 17,
        ResetConnection = 18,
        UserName = 19,
        /// below here: TDS v7.4
        Routing = 20,
    }
}

/// Environment change token [2.2.7.9]
/// A notification of an environment change (for example, database, language, and so on)
#[derive(Debug)]
pub enum TokenEnvChange {
    Database(String, String),
    Language(String, String),
    CharacterSet(String, String),
    RealTimeLogShipping(String, String),
    PacketSize(String, String),
    BeginTransaction([u8; 8]),
    CommitTransaction,
    RollbackTransaction,
    DefectTransaction,
    Routing { host: String, port: u16 },
    ChangeMirror(String),
    Ignored(EnvChangeType),
}

impl TokenEnvChange {
    pub async fn decode<R>(src: &mut R) -> Result<TokenEnvChange>
    where
        R: AsyncRead + Unpin,
    {
        let len = src.read_u16_le().await? as usize;

        // We read all the bytes now, due to whatever environment change tokens
        // we read, they might contain padding zeroes in the end we must
        // discard.
        let mut bytes = vec![0; len];
        src.read_exact(&mut bytes[0..len]).await?;

        let mut buf = Cursor::new(bytes);
        let ty_byte = buf.read_u8().await?;

        let ty = EnvChangeType::try_from(ty_byte)
            .map_err(|_| Error::Protocol(format!("invalid envchange type {:x}", ty_byte).into()))?;

        let token = match ty {
            EnvChangeType::Database | EnvChangeType::PacketSize => {
                let len = buf.read_u8().await? as usize;
                let mut bytes = vec![0; len];

                for item in bytes.iter_mut().take(len) {
                    *item = buf.read_u16_le().await?;
                }

                let new_value = String::from_utf16(&bytes[..]).unwrap();

                let len = buf.read_u8().await? as usize;
                let mut bytes = vec![0; len];

                for item in bytes.iter_mut().take(len) {
                    *item = buf.read_u16_le().await?;
                }

                let old_value = String::from_utf16(&bytes[..]).unwrap();

                TokenEnvChange::Database(new_value, old_value)
            }
            EnvChangeType::SqlCollation => {
                todo!("Not implemented, utf-8 encoding is supported by default since the backend only knows utf-8");
            }
            EnvChangeType::BeginTransaction | EnvChangeType::EnlistDTCTransaction => {
                let len = buf.read_u8().await?;
                assert_eq!(len, 8);

                let mut desc = [0; 8];
                buf.read_exact(&mut desc).await?;

                TokenEnvChange::BeginTransaction(desc)
            }
            EnvChangeType::CommitTransaction => TokenEnvChange::CommitTransaction,
            EnvChangeType::RollbackTransaction => TokenEnvChange::RollbackTransaction,
            EnvChangeType::DefectTransaction => TokenEnvChange::DefectTransaction,
            EnvChangeType::Routing => {
                buf.read_u16_le().await?; // routing data value length
                buf.read_u8().await?; // routing protocol, always 0 (tcp)

                let port = buf.read_u16_le().await?;

                let len = buf.read_u16_le().await? as usize; // hostname string length
                let mut bytes = vec![0; len];

                for item in bytes.iter_mut().take(len) {
                    *item = buf.read_u16_le().await?;
                }

                let host = String::from_utf16(&bytes[..]).unwrap();

                TokenEnvChange::Routing { host, port }
            }
            EnvChangeType::Rtls => {
                let len = buf.read_u8().await? as usize;
                let mut bytes = vec![0; len];

                for item in bytes.iter_mut().take(len) {
                    *item = buf.read_u16_le().await?;
                }

                let mirror_name = String::from_utf16(&bytes[..]).unwrap();

                TokenEnvChange::ChangeMirror(mirror_name)
            }
            ty => TokenEnvChange::Ignored(ty),
        };

        Ok(token)
    }

    pub async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TokenType::EnvChange as u8).await?;

        let mut cache: Vec<u8> = Vec::with_capacity(12);
        let mut buff = BufWriter::new(&mut cache);

        // write envchange type
        match *self {
            TokenEnvChange::Database(_, _) => buff.write_u8(EnvChangeType::Database as u8).await?,
            TokenEnvChange::Language(_, _) => buff.write_u8(EnvChangeType::Language as u8).await?,
            TokenEnvChange::CharacterSet(_, _) => {
                buff.write_u8(EnvChangeType::CharacterSet as u8).await?
            }
            TokenEnvChange::RealTimeLogShipping(_, _) => {
                buff.write_u8(EnvChangeType::Rtls as u8).await?
            }
            TokenEnvChange::PacketSize(_, _) => {
                buff.write_u8(EnvChangeType::PacketSize as u8).await?
            }
            TokenEnvChange::BeginTransaction(_) => {
                buff.write_u8(EnvChangeType::BeginTransaction as u8).await?
            }
            TokenEnvChange::CommitTransaction => {
                buff.write_u8(EnvChangeType::CommitTransaction as u8)
                    .await?
            }
            TokenEnvChange::RollbackTransaction => {
                buff.write_u8(EnvChangeType::RollbackTransaction as u8)
                    .await?
            }
            TokenEnvChange::DefectTransaction => {
                buff.write_u8(EnvChangeType::DefectTransaction as u8)
                    .await?
            }
            TokenEnvChange::Routing { .. } => buff.write_u8(EnvChangeType::Routing as u8).await?,
            TokenEnvChange::ChangeMirror(_) => {
                todo!()
            }
            TokenEnvChange::Ignored(_) => {
                // TODO: return error
                todo!()
            }
        }

        match self {
            TokenEnvChange::Database(new, old)
            | TokenEnvChange::PacketSize(new, old)
            | TokenEnvChange::Language(new, old)
            | TokenEnvChange::CharacterSet(new, old)
            | TokenEnvChange::RealTimeLogShipping(new, old) => {
                buff.write_u8(new.len() as u8).await?;
                encode::write_b_varchar(&mut buff, new).await?;
                buff.write_u8(old.len() as u8).await?;
                encode::write_b_varchar(&mut buff, old).await?;
            }
            //TokenEnvChange::Routing => {}
            _ => todo!("Not sure if we need this"),
        };

        buff.flush().await?;
        dest.write_u16_le(buff.get_ref().len() as u16).await?;
        dest.write_all(buff.get_ref()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Result, TokenEnvChange, TokenType};
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

    #[tokio::test]
    async fn encode_decode_token_envchange_database() -> Result<()> {
        let old_input = "old".to_string();
        let new_input = "new".to_string();
        let input = TokenEnvChange::Database(old_input.clone(), new_input.clone());

        // arrange
        let (inner, outer) = tokio::io::duplex(256);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input.encode(&mut writer).await?;
        writer.flush().await?;

        // decode
        let token_type = reader.read_u8().await?;
        let result = TokenEnvChange::decode(&mut reader).await?;

        // assert
        assert_eq!(token_type, TokenType::EnvChange as u8);
        match result {
            TokenEnvChange::Database(new, old) => {
                assert_eq!(new, new_input);
                assert_eq!(old, old_input);
            }
            _ => std::panic!("unexpected result: {:?}", result),
        }

        Ok(())
    }
}
