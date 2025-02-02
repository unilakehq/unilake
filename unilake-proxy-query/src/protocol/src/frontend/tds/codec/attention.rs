use crate::frontend::{TdsMessage, TdsMessageCodec};
use tokio_util::bytes::BytesMut;
use unilake_common::error::TdsWireResult;

#[derive(Debug)]
pub struct AttentionSignal {}
impl AttentionSignal {}

impl TdsMessageCodec for AttentionSignal {
    fn decode(src: &mut BytesMut) -> TdsWireResult<TdsMessage>
    where
        Self: Sized,
    {
        todo!()
    }

    fn encode(&self, _: &mut BytesMut) -> TdsWireResult<()> {
        todo!()
    }
}
