use crate::{Error, Result};
use enumflags2::{bitflags, BitFlags};
use std::fmt;
use tokio_util::bytes::{Buf, BufMut, BytesMut};

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

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketStatus {
    EndOfMessage = 1 << 0,
    /// [client to server ONLY] (EndOfMessage also required)
    IgnoreEvent = 1 << 1,
    Notification = 1 << 2,
    /// [client to server ONLY] [>= TDSv7.1]
    ResetConnection = 1 << 3,
    /// [client to server ONLY] [>= TDSv7.3]
    ResetConnectionSkipTran = 1 << 4,
}

/// packet header consisting of 8 bytes [2.2.3.1]
#[derive(Debug, Clone, Copy)]
pub struct PacketHeader {
    /// Packet type
    pub ty: PacketType,
    /// Packet Status
    pub is_end_of_message: bool,
    pub is_ignore_event: bool,
    pub is_event_notification: bool,
    pub is_reset_connection: bool,
    pub is_reset_connection_skip_tran: bool,
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
            ty: PacketType::TabularResult,
            is_end_of_message: true,
            length: length as u16,
            spid: 0,
            id,
            window: 0,
            is_event_notification: false,
            is_ignore_event: false,
            is_reset_connection: false,
            is_reset_connection_skip_tran: false,
        }
    }

    pub fn result() -> Self {
        Self {
            ty: PacketType::TabularResult,
            is_end_of_message: true,
            ..Self::new(0, 0)
        }
    }

    pub fn encode(&self, dst: &mut BytesMut) -> Result<()> {
        tracing::debug!(
            message = "Sending packet",
            message_type = self.ty.to_string(),
            message_length = self.length
        );

        dst.put_u8(self.ty as u8);

        let mut flags: BitFlags<PacketStatus> = BitFlags::empty();
        flags.set(PacketStatus::EndOfMessage, self.is_end_of_message);
        flags.set(PacketStatus::IgnoreEvent, self.is_ignore_event);
        flags.set(PacketStatus::Notification, self.is_event_notification);
        flags.set(PacketStatus::ResetConnection, self.is_reset_connection);
        flags.set(
            PacketStatus::ResetConnectionSkipTran,
            self.is_reset_connection_skip_tran,
        );

        dst.put_u8(flags.bits());
        dst.put_u16(self.length);
        dst.put_u16(self.spid);
        dst.put_u8(self.id);
        dst.put_u8(self.window);

        Ok(())
    }

    pub fn decode(src: &mut BytesMut) -> Result<Self> {
        let raw_ty = src.get_u8();
        let ty = PacketType::try_from(raw_ty).map_err(|_| {
            Error::Protocol(format!("header: invalid packet type: {}", raw_ty).into())
        })?;

        let status = BitFlags::from_bits_truncate(src.get_u8());
        let length = src.get_u16();

        Ok(PacketHeader {
            ty,
            is_end_of_message: status.contains(PacketStatus::EndOfMessage),
            is_event_notification: status.contains(PacketStatus::Notification),
            is_ignore_event: status.contains(PacketStatus::IgnoreEvent),
            is_reset_connection: status.contains(PacketStatus::ResetConnection),
            is_reset_connection_skip_tran: status.contains(PacketStatus::ResetConnectionSkipTran),
            length,
            spid: src.get_u16(),
            id: src.get_u8(),
            window: src.get_u8(),
        })
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::bytes::BytesMut;

    use crate::{PacketHeader, PacketType};

    const RAW_BYTES: [u8; 8] = [0x12, 0x01, 0x00, 0x2f, 0x00, 0x00, 0x01, 0x00];

    #[test]
    fn header_raw_decode() {
        let mut bytes = BytesMut::from(&RAW_BYTES[..]);
        let header = PacketHeader::decode(&mut bytes).unwrap();

        assert_eq!(header.length, 47);
        assert_eq!(header.id, 1);
        assert_eq!(header.window, 0);
        assert_eq!(header.spid, 0);
        assert_eq!(header.is_end_of_message, true);
        assert_eq!(header.ty, PacketType::PreLogin);
    }

    #[test]
    fn header_raw_encode() {
        let mut bytes = BytesMut::from(&RAW_BYTES[..]);
        let header = PacketHeader::decode(&mut bytes).unwrap();
        PacketHeader::encode(&header, &mut bytes).unwrap();
        assert_eq!(RAW_BYTES.to_vec(), bytes.to_vec());
    }
}
