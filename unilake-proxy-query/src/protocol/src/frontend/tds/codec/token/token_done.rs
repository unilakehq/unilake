use crate::frontend::{Error, Result, TdsToken, TdsTokenCodec, TdsTokenType};
use enumflags2::{bitflags, BitFlags};
use std::fmt;
use tokio_util::bytes::{Buf, BufMut, BytesMut};

/// Done Token [2.2.7.6]
/// Indicates the completion status of a SQL statement.
#[derive(Debug)]
pub struct TokenDone {
    /// Status
    pub status: BitFlags<DoneStatus>,
    /// The token of the current SQL statement. The token value is provided and controlled by the application layer, which utilizes TDS. The TDS layer does not evaluate the value.
    pub cur_cmd: u16,
    /// The count of rows that were affected by the SQL statement. The value of DoneRowCount is valid if the value of Status includes DONE_COUNT
    pub done_rows: u64,
}

/// Done Token status field [2.2.7.6]
#[bitflags]
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DoneStatus {
    /// DONE_MORE. This DONE message is not the final DONE message in the response. Subsequent data streams to follow.
    More = 1 << 0,
    /// DONE_ERROR. An error occurred on the current SQL statement. A preceding ERROR token SHOULD be sent when this bit is set.
    Error = 1 << 1,
    /// DONE_INXACT. A transaction is in progress
    Inexact = 1 << 2,
    /// DONE_COUNT. The DoneRowCount value is valid. This is used to distinguish between a valid value of 0 for DoneRowCount or just an initialized variable.
    Count = 1 << 4,
    /// DONE_ATTN. The DONE message is a server acknowledgement of a client ATTENTION message.
    Attention = 1 << 5,
    /// Unknown
    RpcInBatch = 1 << 7,
    /// DONE_SRVERROR. Used in place of DONE_ERROR when an error occurred on the current SQL statement, which is severe enough to require the result set, if any, to be discarded.
    SrvError = 1 << 8,
}

impl TokenDone {
    /// DONE_SRVERROR. Used in place of DONE_ERROR when an error occurred on the current SQL statement, which is severe enough to require the result set, if any, to be discarded.
    pub fn new_srv_error(cur_cmd: u16) -> Self {
        TokenDone {
            status: BitFlags::from_flag(DoneStatus::SrvError),
            cur_cmd,
            done_rows: 0,
        }
    }

    /// Unknown
    pub fn new_rpc_in_batch(cur_cmd: u16) -> Self {
        TokenDone {
            status: BitFlags::from_flag(DoneStatus::RpcInBatch),
            cur_cmd,
            done_rows: 0,
        }
    }

    /// DONE_ATTN. The DONE message is a server acknowledgement of a client ATTENTION message.
    pub fn new_attention(cur_cmd: u16) -> Self {
        TokenDone {
            status: BitFlags::from_flag(DoneStatus::Attention),
            cur_cmd,
            done_rows: 0,
        }
    }

    /// DONE_COUNT. The DoneRowCount value is valid. This is used to distinguish between a valid value of 0 for DoneRowCount or just an initialized variable.
    pub fn new_count(cur_cmd: u16, done_rows: u64) -> Self {
        TokenDone {
            status: BitFlags::from_flag(DoneStatus::Count),
            cur_cmd,
            done_rows,
        }
    }

    /// DONE_INXACT. A transaction is in progress
    pub fn new_inexact(cur_cmd: u16) -> Self {
        TokenDone {
            status: BitFlags::from_flag(DoneStatus::More),
            cur_cmd,
            done_rows: 0,
        }
    }

    /// DONE_ERROR. An error occurred on the current SQL statement. A preceding ERROR token SHOULD be sent when this bit is set.
    pub fn new_error(cur_cmd: u16) -> Self {
        TokenDone {
            status: BitFlags::from_flag(DoneStatus::Error),
            cur_cmd,
            done_rows: 0,
        }
    }

    /// DONE_MORE. This DONE message is not the final DONE message in the response. Subsequent data streams to follow.
    pub fn new_more(cur_cmd: u16) -> Self {
        TokenDone {
            status: BitFlags::from_flag(DoneStatus::More),
            cur_cmd,
            done_rows: 0,
        }
    }

    /// DONE_FINAL. This DONE is the final DONE in the request. Also contains the done_rows
    pub fn new_done(cur_cmd: u16) -> Self {
        TokenDone {
            status: BitFlags::empty(),
            cur_cmd,
            done_rows: 0,
        }
    }

    pub fn new_final() -> Self {
        Self::new_done(0)
    }
}

impl TdsTokenCodec for TokenDone {
    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::Done as u8);
        dest.put_u16_le(self.status.bits());
        dest.put_u16_le(self.cur_cmd);
        dest.put_u64_le(self.done_rows);
        Ok(())
    }

    fn decode(src: &mut BytesMut) -> Result<TdsToken> {
        let status = BitFlags::from_bits(src.get_u16_le())
            .map_err(|_| Error::Protocol("token(done): invalid status".into()))?;
        let cur_cmd = src.get_u16_le();
        let done_rows = src.get_u64_le();

        Ok(TdsToken::Done(TokenDone {
            status,
            cur_cmd,
            done_rows,
        }))
    }
}

impl fmt::Display for TokenDone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.done_rows == 0 {
            write!(f, "Done with status {:?}", self.status)
        } else if self.done_rows == 1 {
            write!(f, "Done with status {:?} (1 row left)", self.status)
        } else {
            write!(
                f,
                "Done with status {:?} ({} rows left)",
                self.status, self.done_rows
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::{Result, TdsToken, TdsTokenCodec, TdsTokenType, TokenDone};
    use enumflags2::BitFlags;
    use tokio_util::bytes::{Buf, BytesMut};

    #[test]
    fn encode_decode_token_done_attention() -> Result<()> {
        let input = TokenDone::new_count(1, 127);

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).expect("should be ok");

        // decode
        let token_type = buff.get_u8();
        let result = TokenDone::decode(&mut buff)?;

        // assert
        assert_eq!(token_type, TdsTokenType::Done as u8);
        if let TdsToken::Done(result) = result {
            assert_eq!(result.cur_cmd, input.cur_cmd);
            assert_eq!(result.done_rows, input.done_rows);
            assert_eq!(result.status, input.status);
        }
        Ok(())
    }

    #[test]
    fn encode_decode_token_done_final() -> Result<()> {
        let input = TokenDone {
            done_rows: 128,
            cur_cmd: 1,
            status: BitFlags::empty(),
        };

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).expect("should be ok");

        // decode
        let token_type = buff.get_u8();
        let result = TokenDone::decode(&mut buff)?;

        // assert
        assert_eq!(token_type, TdsTokenType::Done as u8);
        if let TdsToken::Done(result) = result {
            assert_eq!(result.cur_cmd, input.cur_cmd);
            assert_eq!(result.done_rows, input.done_rows);
            assert_eq!(result.status, input.status);
        }

        Ok(())
    }
}
