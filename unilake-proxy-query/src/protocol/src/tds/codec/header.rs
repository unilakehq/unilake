use std::fmt::{self, Write};

use crate::{Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::instrument;

uint_enum! {
    /// the type of the packet [2.2.3.1.1]#[repr(u32)]
    #[repr(u8)]
    pub enum PacketType {
        SQLBatch = 1,
        /// unused
        PreTDSv7Login = 2,
        Rpc = 3,
        TabularResult = 4,
        AttentionSignal = 6,
        BulkLoad = 7,
        /// Federated Authentication Token
        Fat = 8,
        TransactionManagerReq = 14,
        TDSv7Login = 16,
        Sspi = 17,
        PreLogin = 18,
        FederatedAuthenticationInfo = 238,
    }
}

impl fmt::Display for PacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match &self {
            PacketType::SQLBatch => "SQLBatch",
            PacketType::PreTDSv7Login => "PreTDSv7Login",
            PacketType::Rpc => "Rpc",
            PacketType::TabularResult => "TabularResult",
            PacketType::AttentionSignal => "AttentionSignal",
            PacketType::BulkLoad => "BulkLoad",
            PacketType::Fat => "Fat",
            PacketType::TransactionManagerReq => "TransactionManagerReq",
            PacketType::TDSv7Login => "TDSv7Login",
            PacketType::Sspi => "Sspi",
            PacketType::PreLogin => "PreLogin",
            PacketType::FederatedAuthenticationInfo => "FederatedAuthenticationInfo",
        })
    }
}

uint_enum! {
    /// the message state [2.2.3.1.2]
    #[repr(u8)]
    pub enum PacketStatus {
        NormalMessage = 0,
        EndOfMessage = 1,
        /// [client to server ONLY] (EndOfMessage also required)
        IgnoreEvent = 3,
        /// [client to server ONLY] [>= TDSv7.1]
        ResetConnection = 0x08,
        /// [client to server ONLY] [>= TDSv7.3]
        ResetConnectionSkipTran = 0x10,
    }
}

/// packet header consisting of 8 bytes [2.2.3.1]
#[derive(Debug, Clone, Copy)]
pub struct PacketHeader {
    /// Packet type
    pub ty: PacketType,
    /// Packet Status
    pub status: PacketStatus,
    /// [BE] the length of the packet (including the 8 header bytes)
    /// must match the negotiated size sending from client to server [since TDSv7.3] after login
    /// (only if not EndOfMessage)
    pub length: u16,
    /// [BE] the process ID on the server, for debugging purposes only
    pub spid: u16,
    /// packet id
    pub id: u8,
    /// currently unused
    pub window: u8,
}

impl PacketHeader {
    pub fn new(length: usize, id: u8) -> PacketHeader {
        assert!(length <= u16::MAX as usize);
        PacketHeader {
            ty: PacketType::TDSv7Login,
            status: PacketStatus::ResetConnection,
            length: length as u16,
            spid: 0,
            id,
            window: 0,
        }
    }

    pub fn rpc(id: u8) -> Self {
        Self {
            ty: PacketType::Rpc,
            status: PacketStatus::NormalMessage,
            ..Self::new(0, id)
        }
    }

    pub fn pre_login(id: u8) -> Self {
        Self {
            ty: PacketType::PreLogin,
            status: PacketStatus::EndOfMessage,
            ..Self::new(0, id)
        }
    }

    pub fn login(id: u8) -> Self {
        Self {
            ty: PacketType::TDSv7Login,
            status: PacketStatus::EndOfMessage,
            ..Self::new(0, id)
        }
    }

    pub fn batch(id: u8) -> Self {
        Self {
            ty: PacketType::SQLBatch,
            status: PacketStatus::NormalMessage,
            ..Self::new(0, id)
        }
    }

    pub fn set_status(&mut self, status: PacketStatus) {
        self.status = status;
    }

    #[instrument(skip(dst))]
    pub async fn encode<W>(&self, dst: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        tracing::debug!(
            message = "Sending packet",
            message_type = self.ty as u8,
            message_length = self.length
        );

        dst.write_u8(self.ty as u8).await?;
        dst.write_u8(self.status as u8).await?;
        dst.write_u16_le(self.length).await?;
        dst.write_u16_le(self.spid).await?;
        dst.write_u8(self.id).await?;
        dst.write_u8(self.window).await?;

        Ok(())
    }

    pub async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let raw_ty = src.read_u8().await?;
        let ty = PacketType::try_from(raw_ty).map_err(|_| {
            Error::Protocol(format!("header: invalid packet type: {}", raw_ty).into())
        })?;

        let status = PacketStatus::try_from(src.read_u8().await?)
            .map_err(|_| Error::Protocol("header: invalid packet status".into()))?;

        let length = src.read_u16_le().await?;

        tracing::debug!(
            message = "Receiving packet",
            message_type = ty.to_string(),
            message_length = length
        );

        Ok(PacketHeader {
            ty,
            status,
            length,
            spid: src.read_u16_le().await?,
            id: src.read_u8().await?,
            window: src.read_u8().await?,
        })
    }
}
