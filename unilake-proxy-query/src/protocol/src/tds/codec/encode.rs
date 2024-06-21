use crate::Result;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub async fn write_us_varchar<R>(dest: &mut R, s: &String) -> Result<()>
where
    R: AsyncWrite + Unpin,
{
    dest.write_u16_le(s.len() as u16).await?;
    dest.write_all(s.as_bytes()).await?;
    Ok(())
}

pub async fn write_b_varchar<R>(dest: &mut R, s: &String) -> Result<()>
where
    R: AsyncWrite + Unpin,
{
    dest.write_u8(s.len() as u8).await?;
    dest.write_all(s.as_bytes()).await?;
    Ok(())
}
