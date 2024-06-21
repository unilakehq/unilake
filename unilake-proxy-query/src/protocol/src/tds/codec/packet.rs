// TODO: this, note you can send multiple tds_tokens in a single response, with one header

use crate::tds::codec::header::{PacketHeader, PacketStatus};
use crate::*;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct Packet {
    pub header: PacketHeader,
    pub payload: Vec<TdsToken>,
}

pub enum TdsToken {
    Done(TokenDone),
    EnvChange(TokenEnvChange),
    Error(TokenError),
    Info(TokenInfo),
    Order(TokenOrder),
}

impl Packet {
    pub(crate) fn new(header: PacketHeader, payload: Vec<TdsToken>) -> Self {
        Self { header, payload }
    }

    pub(crate) fn is_last(&self) -> bool {
        self.header.status == PacketStatus::EndOfMessage
    }

    pub(crate) fn into_parts(self) -> (PacketHeader, Vec<TdsToken>) {
        (self.header, self.payload)
    }

    pub async fn encode<W>(&self, dst: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        todo!()
    }

    pub async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        todo!()
    }
}
