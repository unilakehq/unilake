use tokio_util::bytes::{BufMut, BytesMut};

/// Variable length-prefixed token [2.2.5.2.2]
pub(crate) fn encode(dest: &mut BytesMut, data: Option<&Vec<u8>>) {
    match data {
        // Encoding a NULL
        None => {
            dest.put_u16_le(0xffff);
        }
        Some(data) => {
            let len = data.len();

            // Fixed size
            if len < 0xffff {
                // Encode the length first
                dest.put_u16_le(len as u16);

                // Encode the actual data
                for &byte in data {
                    dest.put_u8(byte);
                }
            } else {
                // Unknown size, length-prefixed blobs
                dest.put_u64_le(len as u64);

                let mut data_left = len;
                let mut offset = 0;

                while data_left > 0 {
                    // Determine the size of the next chunk
                    let chunk_size = std::cmp::min(data_left, 0xffff_ffff) as u32;
                    dest.put_u32_le(chunk_size);

                    // Encode the chunk data
                    for &byte in &data[offset..offset + chunk_size as usize] {
                        dest.put_u8(byte);
                    }

                    data_left -= chunk_size as usize;
                    offset += chunk_size as usize;
                }

                // Write a zero-length chunk as a sentinel
                dest.put_u32_le(0);
            }
        }
    }
}
