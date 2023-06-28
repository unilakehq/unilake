use crate::{Error, Result};
use byteorder::{ByteOrder, LittleEndian};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Represent a sql Decimal / Numeric type. It is stored in a i128 and has a
/// maximum precision of 38 decimals.
///
/// A recommended way of dealing with numeric values is by enabling the
/// `rust_decimal` feature and using its `Decimal` type instead.
#[derive(Copy, Clone, Debug)]
pub struct Numeric {
    value: i128,
    scale: u8,
}

impl Numeric {
    /// Creates a new Numeric value.
    ///
    /// # Panic
    /// It will panic if the scale exceed 37.
    pub fn new_with_scale(value: i128, scale: u8) -> Self {
        // scale cannot exceed 37 since a
        // max precision of 38 is possible here.
        assert!(scale < 38);
        Numeric { value, scale }
    }

    /// Extract the decimal part.
    pub fn dec_part(self) -> i128 {
        let scale = self.pow_scale();
        self.value - (self.value / scale) * scale
    }

    /// Extract the integer part.
    pub fn int_part(self) -> i128 {
        self.value / self.pow_scale()
    }

    #[inline]
    fn pow_scale(self) -> i128 {
        10i128.pow(self.scale as u32)
    }

    /// The scale (where is the decimal point) of the value.
    #[inline]
    pub fn scale(self) -> u8 {
        self.scale
    }

    /// The internal integer value
    #[inline]
    pub fn value(self) -> i128 {
        self.value
    }

    /// The precision of the `Number` as a number of digits.
    pub fn precision(self) -> u8 {
        let mut result = 0;
        let mut n = self.int_part();

        while n != 0 {
            n /= 10;
            result += 1;
        }

        if result == 0 {
            1 + self.scale()
        } else {
            result + self.scale()
        }
    }

    pub(crate) fn len(self) -> u8 {
        match self.precision() {
            1..=9 => 5,
            10..=19 => 9,
            20..=28 => 13,
            _ => 17,
        }
    }

    pub(crate) async fn decode<R>(src: &mut R, scale: u8) -> Result<Option<Self>>
    where
        R: AsyncRead + Unpin,
    {
        fn decode_d128(buf: &[u8]) -> u128 {
            let low_part = LittleEndian::read_u64(&buf[0..]) as u128;

            if !buf[8..].iter().any(|x| *x != 0) {
                return low_part;
            }

            let high_part = match buf.len() {
                12 => LittleEndian::read_u32(&buf[8..]) as u128,
                16 => LittleEndian::read_u64(&buf[8..]) as u128,
                _ => unreachable!(),
            };

            let high_part = high_part * (u64::MAX as u128 + 1);
            low_part + high_part
        }

        let len = src.read_u8().await?;

        if len == 0 {
            Ok(None)
        } else {
            let sign = match src.read_u8().await? {
                0 => -1i128,
                1 => 1i128,
                _ => return Err(Error::Protocol("decimal: invalid sign".into())),
            };

            let value = match len {
                5 => src.read_u32_le().await? as i128 * sign,
                9 => src.read_u64_le().await? as i128 * sign,
                13 => {
                    let mut bytes = [0u8; 12]; //u96
                    for item in &mut bytes {
                        *item = src.read_u8().await?;
                    }
                    decode_d128(&bytes) as i128 * sign
                }
                17 => {
                    let mut bytes = [0u8; 16];
                    for item in &mut bytes {
                        *item = src.read_u8().await?;
                    }
                    decode_d128(&bytes) as i128 * sign
                }
                x => {
                    return Err(Error::Protocol(
                        format!("decimal/numeric: invalid length of {} received", x).into(),
                    ))
                }
            };

            Ok(Some(Numeric::new_with_scale(value, scale)))
        }
    }

    pub(crate) async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(self.len()).await?;

        if self.value < 0 {
            dest.write_u8(0).await?;
        } else {
            dest.write_u8(1).await?;
        }

        let value = self.value().abs();

        match self.len() {
            5 => dest.write_u32_le(value as u32).await?,
            9 => dest.write_u64_le(value as u64).await?,
            13 => {
                dest.write_u64_le(value as u64).await?;
                dest.write_u32_le((value >> 64) as u32).await?
            }
            _ => dest.write_u128_le(value as u128).await?,
        }

        Ok(())
    }
}
