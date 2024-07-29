use rand::Rng;
use tokio_util::bytes::BytesMut;

pub(crate) trait ReadAndAdvance {
    fn read_and_advance(&mut self, max_bytes: usize) -> (usize, Vec<u8>);
}

impl ReadAndAdvance for BytesMut {
    fn read_and_advance(&mut self, max_bytes: usize) -> (usize, Vec<u8>) {
        let bytes_to_read = self.len().min(max_bytes);
        (bytes_to_read, self.split_to(bytes_to_read).to_vec())
    }
}

pub fn generate_random_nonce() -> [u8; 32] {
    rand::thread_rng().gen::<[u8; 32]>()
}
