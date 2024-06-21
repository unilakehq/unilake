use crate::Result;
use tokio::io::{AsyncRead, AsyncReadExt};

pub async fn read_us_varchar<R>(src: &mut R) -> Result<String>
where
    R: AsyncRead + Unpin,
{
    let lines = src.read_u16_le().await?;
    return if lines > 0 {
        let mut chars = Vec::with_capacity(lines as usize);
        for _ in 0..lines {
            chars.push(src.read_u8().await?);
        }
        Ok(String::from_utf8(chars).unwrap())
    } else {
        Ok(String::new())
    };
}

pub async fn read_b_varchar<R>(src: &mut R) -> Result<String>
where
    R: AsyncRead + Unpin,
{
    let lines = src.read_u8().await?;
    return if lines > 0 {
        let mut chars = Vec::with_capacity(lines as usize);
        for _ in 0..lines {
            chars.push(src.read_u8().await?);
        }
        Ok(String::from_utf8(chars).unwrap())
    } else {
        Ok(String::new())
    };
}
