use rand::Rng;
use tokio_util::bytes::{Buf, BytesMut};

use crate::error::TdsWireResult;

pub(crate) trait ReadAndAdvance {
    fn read_and_advance(&mut self, max_bytes: usize) -> (usize, Vec<u8>);
    fn put_and_advance(&mut self, target: &mut [u8]) -> TdsWireResult<()>;
}

impl ReadAndAdvance for BytesMut {
    fn read_and_advance(&mut self, max_bytes: usize) -> (usize, Vec<u8>) {
        let bytes_to_read = self.len().min(max_bytes);
        (bytes_to_read, self.split_to(bytes_to_read).to_vec())
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
