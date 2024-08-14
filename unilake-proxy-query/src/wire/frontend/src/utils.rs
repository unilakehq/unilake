use rand::Rng;
use tokio_util::bytes::{Buf, BytesMut};

use crate::error::TdsWireResult;

pub(crate) trait ReadAndAdvance {
    fn read_and_advance(&mut self, max_bytes: usize) -> (usize, BytesMut);
    fn put_and_advance(&mut self, target: &mut [u8]) -> TdsWireResult<()>;
}

impl ReadAndAdvance for BytesMut {
    fn read_and_advance(&mut self, max_bytes: usize) -> (usize, BytesMut) {
        let bytes_to_read = std::cmp::min(max_bytes, self.remaining());
        let mut buf = BytesMut::with_capacity(bytes_to_read);

        // Copy the data from the source to the buffer
        buf.extend_from_slice(&self[..bytes_to_read]);

        // Advance the internal cursor by the number of bytes read
        self.advance(bytes_to_read);

        (bytes_to_read, buf)
    }

    fn put_and_advance(&mut self, target: &mut [u8]) -> TdsWireResult<()> {
        let bytes_to_write = self.len().min(target.len());
        target[..bytes_to_write].copy_from_slice(&self.split_to(bytes_to_write));
        Ok(())
    }
}

pub fn generate_random_nonce() -> [u8; 32] {
    rand::thread_rng().gen::<[u8; 32]>()
}
