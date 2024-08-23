use tokio_util::bytes::{Buf, BufMut, BytesMut};

/// Variable length-prefixed token [2.2.5.2.2]
pub(crate) fn encode(dest: &mut BytesMut, type_length: &usize, data: Option<&String>) {
    match data {
        // Encoding a NULL
        None => {
            dest.put_u16_le(0xffff);
        }
        Some(data) => {
            // Fixed size
            let mut data = BytesMut::from_iter(data.encode_utf16().flat_map(|b| b.to_le_bytes()));

            if *type_length < 0xffff {
                // Encode the length first
                dest.put_u16_le(data.len() as u16);

                // Encode the actual data
                dest.extend_from_slice(&data);
            } else {
                // Unknown size, length-prefixed blobs
                dest.put_u64_le(data.len() as u64);

                while data.has_remaining() {
                    // Determine the size of the next chunk
                    let chunk_size = std::cmp::min(data.len(), 4035);
                    dest.put_u32_le(chunk_size as u32);

                    // Encode the chunk data
                    dest.extend_from_slice(&data.split_to(chunk_size));
                }

                // Write a zero-length chunk as a sentinel
                dest.put_u32_le(0);
            }
        }
    }
}
