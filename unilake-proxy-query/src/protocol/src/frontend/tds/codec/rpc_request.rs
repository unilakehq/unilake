// MS-TDS: [2.2.6.6]
use crate::frontend::tds::codec::decode::read_b_varchar;
use crate::frontend::{ColumnData, TdsMessage, TdsMessageCodec, TypeInfo};
use tokio_util::bytes::{Buf, BytesMut};
use unilake_common::error::TdsWireResult;

uint_enum! {
    #[repr(u16)]
    pub enum ProcedureType {
        SpCursor = 1,
        SpCursorOpen = 2,
        SpCursorPrepare = 3,
        SpCursorExecute = 4,
        SpCursorPrepExec = 5,
        SpCursorUnprepare = 6,
        SpCursorFetch = 7,
        SpCursorOption = 8,
        SpCursorClose = 9,
        SpExecuteSql = 10,
        SpPrepare = 11,
        SpExecute = 12,
        SpPrepExec = 13,
        SpPrepExecRpc = 14,
        SpUnprepare = 15,
    }
}

#[derive(Debug)]
pub struct RpcRequest {
    outstanding_requests: u32,
    procedure_type: ProcedureType,
    parameters: Vec<RpcParameter>,
}

#[derive(Debug)]
pub struct RpcParameter {
    name: String,
    status: u8,
    type_info: TypeInfo,
    value: ColumnData,
}

impl TdsMessageCodec for RpcRequest {
    fn decode(src: &mut BytesMut) -> TdsWireResult<TdsMessage>
    where
        Self: Sized,
    {
        let _total_length = src.get_u32_le();

        // Header
        let _header_length = src.get_u32_le();
        // let mut src_header = src.split_to(header_length as usize);
        let _rpc_header_type = src.get_u16_le();
        let _transaction_descriptor = src.get_f64_le();
        let outstanding_requests = src.get_u32_le();

        // Procedure Name Length
        let _procedure_name_length = src.get_u16_le();

        // Stored Procedure ID
        let procedure_type = ProcedureType::try_from(src.get_u16_le()).map_err(|_| {
            unilake_common::error::Error::Protocol("invalid procedure type".to_string())
        })?;

        // Options Flag
        let _options_flag = src.get_u16_le();

        // Parameters
        let mut parameters = Vec::<RpcParameter>::new();
        while src.has_remaining() {
            let name = read_b_varchar(src)?;
            let status = src.get_u8();
            let type_info = TypeInfo::decode(src)?;
            let value = ColumnData::decode(src, &type_info)?;

            parameters.push(RpcParameter {
                name,
                status,
                type_info,
                value,
            });
        }

        Ok(TdsMessage::RemoteProcedureCall(RpcRequest {
            outstanding_requests,
            procedure_type,
            parameters,
        }))
    }

    fn encode(&self, _: &mut BytesMut) -> TdsWireResult<()> {
        unimplemented!("Encode on RpcRequest is not a server implementation")
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::tds::codec::rpc_request::RpcRequest;
    use crate::frontend::{TdsMessage, TdsMessageCodec};
    use tokio_util::bytes::{Buf, BytesMut};

    const RAW_BYTES: &[u8] = &[
        0x16, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xff, 0xff, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xe7, 0x40, 0x1f, 0x09, 0x04, 0xd0, 0x00, 0x34, 0x30, 0x00, 0x75, 0x00, 0x73, 0x00, 0x65,
        0x00, 0x20, 0x00, 0x41, 0x00, 0x64, 0x00, 0x76, 0x00, 0x65, 0x00, 0x6e, 0x00, 0x74, 0x00,
        0x75, 0x00, 0x72, 0x00, 0x65, 0x00, 0x57, 0x00, 0x6f, 0x00, 0x72, 0x00, 0x6b, 0x00, 0x73,
        0x00, 0x44, 0x00, 0x57, 0x00, 0x32, 0x00, 0x30, 0x00, 0x32, 0x00, 0x32, 0x00,
    ];

    #[test]
    fn decode_example() {
        let mut buf = BytesMut::from(RAW_BYTES);
        let msg = RpcRequest::decode(&mut buf).unwrap();

        assert_eq!(buf.remaining(), 0);
        if let TdsMessage::RemoteProcedureCall(rpc) = msg {
            assert_eq!(rpc.parameters.iter().count(), 1);
        } else {
            panic!("Incorrect return type found")
        }
    }
}
