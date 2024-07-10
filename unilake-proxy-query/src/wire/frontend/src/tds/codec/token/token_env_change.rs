//TODO: fix unwraps in this file for proper error handling
use crate::tds::codec::encode;
use crate::utils::ReadAndAdvance;
use crate::{Error, Result, TdsToken, TdsTokenCodec, TdsTokenType};
use std::fmt::Debug;
use std::io::{Cursor, Read};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

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

impl TdsTokenCodec for TokenEnvChange {
    fn decode(src: &mut BytesMut) -> Result<TdsToken> {
        let len = src.get_u16_le() as usize;

        // We read all the bytes now, due to whatever environment change tokens
        // we read, they might contain padding zeroes in the end we must
        // discard.
        let (_, bytes) = src.read_and_advance(len);

        let mut buf = Cursor::new(bytes);
        let ty_byte = buf.get_u8();

        let ty = EnvChangeType::try_from(ty_byte)
            .map_err(|_| Error::Protocol(format!("invalid envchange type {:x}", ty_byte).into()))?;

        let token = match ty {
            EnvChangeType::Database | EnvChangeType::PacketSize => {
                let len = buf.get_u8() as usize;
                let mut bytes = vec![0; len];

                for item in bytes.iter_mut().take(len) {
                    *item = buf.get_u16_le();
                }

                let new_value = String::from_utf16(&bytes[..]).unwrap();

                let len = buf.get_u8() as usize;
                let mut bytes = vec![0; len];

                for item in bytes.iter_mut().take(len) {
                    *item = buf.get_u16_le();
                }

                let old_value = String::from_utf16(&bytes[..]).unwrap();

                TokenEnvChange::Database(new_value, old_value)
            }
            EnvChangeType::SqlCollation => {
                todo!("Not implemented, utf-8 encoding is supported by default since the backend only knows utf-8");
            }
            EnvChangeType::BeginTransaction | EnvChangeType::EnlistDTCTransaction => {
                let len = buf.get_u8();
                assert_eq!(len, 8);

                let mut desc = [0; 8];
                buf.read_exact(&mut desc);

                TokenEnvChange::BeginTransaction(desc)
            }
            EnvChangeType::CommitTransaction => TokenEnvChange::CommitTransaction,
            EnvChangeType::RollbackTransaction => TokenEnvChange::RollbackTransaction,
            EnvChangeType::DefectTransaction => TokenEnvChange::DefectTransaction,
            EnvChangeType::Routing => {
                buf.get_u16_le(); // routing data value length
                buf.get_u8(); // routing protocol, always 0 (tcp)

                let port = buf.get_u16_le();

                let len = buf.get_u16_le() as usize; // hostname string length
                let mut bytes = vec![0; len];

                for item in bytes.iter_mut().take(len) {
                    *item = buf.get_u16_le();
                }

                let host = String::from_utf16(&bytes[..]).unwrap();

                TokenEnvChange::Routing { host, port }
            }
            EnvChangeType::Rtls => {
                let len = buf.get_u8() as usize;
                let mut bytes = vec![0; len];

                for item in bytes.iter_mut().take(len) {
                    *item = buf.get_u16_le();
                }

                let mirror_name = String::from_utf16(&bytes[..]).unwrap();

                TokenEnvChange::ChangeMirror(mirror_name)
            }
            ty => TokenEnvChange::Ignored(ty),
        };

        Ok(TdsToken::EnvChange(token))
    }

    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::EnvChange as u8);

        let mut buff = BytesMut::new();

        // write envchange type
        match *self {
            TokenEnvChange::Database(_, _) => buff.put_u8(EnvChangeType::Database as u8),
            TokenEnvChange::Language(_, _) => buff.put_u8(EnvChangeType::Language as u8),
            TokenEnvChange::CharacterSet(_, _) => buff.put_u8(EnvChangeType::CharacterSet as u8),
            TokenEnvChange::RealTimeLogShipping(_, _) => buff.put_u8(EnvChangeType::Rtls as u8),
            TokenEnvChange::PacketSize(_, _) => buff.put_u8(EnvChangeType::PacketSize as u8),
            TokenEnvChange::BeginTransaction(_) => {
                buff.put_u8(EnvChangeType::BeginTransaction as u8)
            }
            TokenEnvChange::CommitTransaction => {
                buff.put_u8(EnvChangeType::CommitTransaction as u8)
            }
            TokenEnvChange::RollbackTransaction => {
                buff.put_u8(EnvChangeType::RollbackTransaction as u8)
            }
            TokenEnvChange::DefectTransaction => {
                buff.put_u8(EnvChangeType::DefectTransaction as u8)
            }
            TokenEnvChange::Routing { .. } => buff.put_u8(EnvChangeType::Routing as u8),
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
                buff.put_u8(new.len() as u8);
                encode::write_b_varchar(&mut buff, new);
                buff.put_u8(old.len() as u8);
                encode::write_b_varchar(&mut buff, old);
            }
            //TokenEnvChange::Routing => {}
            _ => todo!("Not sure if we need this"),
        };

        dest.put_u16_le(buff.len() as u16);
        dest.put(buff);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::bytes::{Buf, BytesMut};

    use crate::{Result, TdsToken, TdsTokenCodec, TdsTokenType, TokenEnvChange};

    #[test]
    fn encode_decode_token_envchange_database() -> Result<()> {
        let old_input = "old".to_string();
        let new_input = "new".to_string();
        let input = TokenEnvChange::Database(old_input.clone(), new_input.clone());

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff);

        // decode
        let token_type = buff.get_u8();
        let result = TokenEnvChange::decode(&mut buff).unwrap();

        // assert
        assert_eq!(token_type, TdsTokenType::EnvChange as u8);
        if let TdsToken::EnvChange(result) = result {
            match result {
                TokenEnvChange::Database(new, old) => {
                    assert_eq!(new, new_input);
                    assert_eq!(old, old_input);
                }
                _ => std::panic!("unexpected result: {:?}", result),
            }
        }
        Ok(())
    }
}
