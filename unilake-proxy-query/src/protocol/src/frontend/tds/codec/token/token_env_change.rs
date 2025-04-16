use crate::frontend::tds::codec::token::{TdsToken, TdsTokenCodec, TdsTokenType};
use crate::frontend::tds::codec::{decode, encode};
use crate::frontend::tds::collation::Collation;
use crate::frontend::utils::ReadAndAdvance;
use std::fmt::{self, Debug};
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use unilake_common::error::Result;
use unilake_common::error_code::ErrorCode;

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

impl fmt::Display for EnvChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match &self {
            EnvChangeType::Database => "Database",
            EnvChangeType::Language => "Language",
            EnvChangeType::CharacterSet => "CharacterSet",
            EnvChangeType::PacketSize => "PacketSize",
            EnvChangeType::UnicodeDataSortingLID => "UnicodeDataSortingLID",
            EnvChangeType::UnicodeDataSortingCFL => "UnicodeDataSortingCFL",
            EnvChangeType::SqlCollation => "SqlCollation",
            EnvChangeType::BeginTransaction => "BeginTransaction",
            EnvChangeType::CommitTransaction => "CommitTransaction",
            EnvChangeType::RollbackTransaction => "RollbackTransaction",
            EnvChangeType::EnlistDTCTransaction => "EnlistDTCTransaction",
            EnvChangeType::DefectTransaction => "DefectTransaction",
            EnvChangeType::Rtls => "Rtls",
            EnvChangeType::PromoteTransaction => "PromoteTransaction",
            EnvChangeType::TransactionManagerAddress => "TransactionManagerAddress",
            EnvChangeType::TransactionEnded => "TransactionEnded",
            EnvChangeType::ResetConnection => "ResetConnection",
            EnvChangeType::UserName => "UserName",
            EnvChangeType::Routing => "Routing",
        })
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
    SqlCollation(Option<Collation>, Option<Collation>),
    BeginTransaction([u8; 8]),
    CommitTransaction,
    RollbackTransaction,
    DefectTransaction,
    Routing { host: String, port: u16 },
    ChangeMirror(String),
    Ignored(EnvChangeType),
    ResetConnection,
}

impl TokenEnvChange {
    pub fn new_database_change(old: String, new: String) -> Self {
        Self::Database(old, new)
    }
    pub fn new_language_change(old: String, new: String) -> Self {
        Self::Language(old, new)
    }
    pub fn new_collation_change(old: Option<Collation>, new: Option<Collation>) -> Self {
        Self::SqlCollation(old, new)
    }
    pub fn new_packet_size_change(old: String, new: String) -> Self {
        Self::PacketSize(old, new)
    }
    pub fn new_reset_connection_ack() -> Self {
        Self::ResetConnection
    }
}

impl TdsTokenCodec for TokenEnvChange {
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
            TokenEnvChange::SqlCollation(_, _) => buff.put_u8(EnvChangeType::SqlCollation as u8),
            TokenEnvChange::ChangeMirror(_) => {
                todo!()
            }
            TokenEnvChange::Ignored(t) => {
                tracing::warn!(
                    message = "Encoding ignored env change type",
                    token_type = t.to_string()
                );
            }
            TokenEnvChange::ResetConnection => buff.put_u8(EnvChangeType::ResetConnection as u8),
        }

        // write changed data
        match self {
            TokenEnvChange::Database(old, new)
            | TokenEnvChange::PacketSize(old, new)
            | TokenEnvChange::Language(old, new)
            | TokenEnvChange::CharacterSet(old, new)
            | TokenEnvChange::RealTimeLogShipping(old, new) => {
                encode::write_b_varchar(&mut buff, new)?;
                encode::write_b_varchar(&mut buff, old)?;
            }
            TokenEnvChange::SqlCollation(old, new) => {
                // todo: improve collation use and encoding, this should be a collation object and not raw bytes
                if let Some(new) = new {
                    buff.put_u8(5);
                    buff.put_u16_le(new.codepage);
                    buff.put_u16_le(new.flags);
                    buff.put_u8(new.charset_id);
                } else {
                    buff.put_u8(0);
                }
                if let Some(old) = old {
                    buff.put_u8(5);
                    buff.put_u16_le(old.codepage);
                    buff.put_u16_le(old.flags);
                    buff.put_u8(old.charset_id);
                } else {
                    buff.put_u8(0);
                }
            }
            _ => {
                buff.put_u8(0);
                buff.put_u8(0);
            }
        };

        dest.put_u16_le(buff.len() as u16);
        dest.extend_from_slice(&buff);

        Ok(())
    }

    fn decode(src: &mut BytesMut) -> Result<TdsToken> {
        let len = src.get_u16_le() as usize;

        // We read all the bytes now, due to whatever environment change tokens
        // we read, they might contain padding zeroes in the end we must
        // discard.
        let mut buf = src.split_to(len);
        let ty_byte = buf.get_u8();
        let ty = EnvChangeType::try_from(ty_byte).map_err(|_| {
            ErrorCode::TdsInvalidEnvChangeType(format!("invalid envchange type {:x}", ty_byte))
        })?;

        let token = match ty {
            EnvChangeType::Database | EnvChangeType::PacketSize => {
                let new_value = decode::read_b_varchar(&mut buf)?;
                let old_value = decode::read_b_varchar(&mut buf)?;

                TokenEnvChange::Database(new_value, old_value)
            }
            EnvChangeType::BeginTransaction | EnvChangeType::EnlistDTCTransaction => {
                let len = buf.get_u8();
                assert_eq!(len, 8);

                let mut desc = [0; 8];
                buf.put_and_advance(&mut desc)?;

                TokenEnvChange::BeginTransaction(desc)
            }
            EnvChangeType::CommitTransaction => TokenEnvChange::CommitTransaction,
            EnvChangeType::RollbackTransaction => TokenEnvChange::RollbackTransaction,
            EnvChangeType::DefectTransaction => TokenEnvChange::DefectTransaction,
            EnvChangeType::Routing => {
                buf.get_u16_le(); // routing data value length
                buf.get_u8(); // routing protocol, always 0 (tcp)

                let port = buf.get_u16_le();
                let host = decode::read_us_varchar(&mut buf)?;

                TokenEnvChange::Routing { host, port }
            }
            EnvChangeType::Rtls => {
                let mirror_name = decode::read_b_varchar(&mut buf)?;
                TokenEnvChange::ChangeMirror(mirror_name)
            }
            ty => TokenEnvChange::Ignored(ty),
        };

        Ok(TdsToken::EnvChange(token))
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::tds::codec::token::{
        TdsToken, TdsTokenCodec, TdsTokenType, TokenEnvChange,
    };
    use tokio_util::bytes::{Buf, BytesMut};
    use unilake_common::error::Result;

    #[test]
    fn encode_decode_token_envchange_database() -> Result<()> {
        let old_input = "old".to_string();
        let new_input = "new".to_string();
        let input = TokenEnvChange::Database(old_input.clone(), new_input.clone());

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff)?;

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
