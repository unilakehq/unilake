use crate::frontend::tds::codec::{TdsMessage, TdsMessageCodec};
use tokio_util::bytes::BytesMut;
use unilake_common::error::Result;

#[derive(Debug)]
pub struct AttentionSignal {}
impl AttentionSignal {}

impl TdsMessageCodec for AttentionSignal {
    fn decode(src: &mut BytesMut) -> Result<TdsMessage>
    where
        Self: Sized,
    {
        todo!()
    }

    fn encode(&self, _: &mut BytesMut) -> Result<()> {
        todo!()
    }
}
