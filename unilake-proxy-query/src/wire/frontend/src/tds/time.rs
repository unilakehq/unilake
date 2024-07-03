use crate::Result;
use byteorder::{ByteOrder, LittleEndian};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// A presentation of `date` type in the server.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Date(u32);

impl Date {
    #[inline]
    /// Construct a new `Date`
    ///
    /// # Panics
    /// max value of 3 bytes (`u32::max_value() > 8`)
    pub fn new(days: u32) -> Date {
        assert_eq!(days >> 24, 0);
        Date(days)
    }

    #[inline]
    /// The number of days from 1st of January, year 1.
    pub fn days(self) -> u32 {
        self.0
    }

    pub(crate) async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let mut bytes = [0u8; 4];
        src.read_exact(&mut bytes[..3]).await?;
        Ok(Self::new(LittleEndian::read_u32(&bytes)))
    }

    pub(crate) async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let mut tmp = [0u8; 4];
        LittleEndian::write_u32(&mut tmp, self.days());
        assert_eq!(tmp[3], 0);
        dest.write_all(&tmp[0..3]).await?;
        Ok(())
    }
}

/// A presentation of `datetime` type in the server.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct DateTime {
    days: i32,
    seconds_fragments: u32,
}

impl DateTime {
    /// Construct a new `DateTime` instance.
    pub fn new(days: i32, seconds_fragments: u32) -> Self {
        Self {
            days,
            seconds_fragments,
        }
    }

    /// Days since 1st of January, 1900 (including the negative range until 1st
    /// of January, 1753).
    pub fn days(self) -> i32 {
        self.days
    }

    /// 1/300 of a second, so a value of 300 equals 1 second (since midnight).
    pub fn seconds_fragments(self) -> u32 {
        self.seconds_fragments
    }

    pub(crate) async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let days = src.read_i32_le().await?;
        let seconds_fragments = src.read_u32_le().await?;

        Ok(Self {
            days,
            seconds_fragments,
        })
    }

    pub(crate) async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_i32_le(self.days).await?;
        dest.write_u32_le(self.seconds_fragments).await?;

        Ok(())
    }
}

/// A presentation of `smalldatetime` type in the server.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SmallDateTime {
    days: u16,
    seconds_fragments: u16,
}

impl SmallDateTime {
    /// Construct a new `SmallDateTime` instance.
    pub fn new(days: u16, seconds_fragments: u16) -> Self {
        Self {
            days,
            seconds_fragments,
        }
    }
    /// Days since 1st of January, 1900.
    pub fn days(self) -> u16 {
        self.days
    }

    /// 1/300 of a second, so a value of 300 equals 1 second (since midnight)
    pub fn seconds_fragments(self) -> u16 {
        self.seconds_fragments
    }

    pub(crate) async fn decode<R>(src: &mut R) -> crate::Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let days = src.read_u16_le().await?;
        let seconds_fragments = src.read_u16_le().await?;

        Ok(Self {
            days,
            seconds_fragments,
        })
    }

    pub(crate) async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u16_le(self.days).await?;
        dest.write_u16_le(self.seconds_fragments).await?;

        Ok(())
    }
}

/// A presentation of `time` type in the server.
#[derive(Copy, Clone, Debug)]
pub struct Time {
    increments: u64,
    scale: u8,
}

impl PartialEq for Time {
    fn eq(&self, t: &Time) -> bool {
        self.increments as f64 / 10f64.powi(self.scale as i32)
            == t.increments as f64 / 10f64.powi(t.scale as i32)
    }
}

impl Time {
    /// Construct a new `Time`
    pub fn new(increments: u64, scale: u8) -> Self {
        Self { increments, scale }
    }

    #[inline]
    /// Number of 10^-n second increments since midnight, where `n` is defined
    /// in [`scale`].
    ///
    /// [`scale`]: #method.scale
    pub fn increments(self) -> u64 {
        self.increments
    }

    #[inline]
    /// The accuracy of the increments.
    pub fn scale(self) -> u8 {
        self.scale
    }

    #[inline]
    /// Length of the field in number of bytes.
    pub(crate) fn len(self) -> Result<u8> {
        Ok(match self.scale {
            0..=2 => 3,
            3..=4 => 4,
            5..=7 => 5,
            _ => {
                return Err(crate::Error::Protocol(
                    format!("time: invalid scale {}", self.scale).into(),
                ))
            }
        })
    }

    pub(crate) async fn decode<R>(src: &mut R, n: usize, rlen: usize) -> Result<Time>
    where
        R: AsyncRead + Unpin,
    {
        let val = match (n, rlen) {
            (0..=2, 3) => {
                let hi = src.read_u16_le().await? as u64;
                let lo = src.read_u8().await? as u64;

                hi | lo << 16
            }
            (3..=4, 4) => src.read_u32_le().await? as u64,
            (5..=7, 5) => {
                let hi = src.read_u32_le().await? as u64;
                let lo = src.read_u8().await? as u64;

                hi | lo << 32
            }
            _ => {
                return Err(crate::Error::Protocol(
                    format!("time: invalid length {}", n).into(),
                ))
            }
        };

        Ok(Time {
            increments: val,
            scale: n as u8,
        })
    }

    pub(crate) async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        match self.len()? {
            3 => {
                assert_eq!(self.increments >> 24, 0);
                dest.write_u16_le(self.increments as u16).await?;
                dest.write_u8((self.increments >> 16) as u8).await?;
            }
            4 => {
                assert_eq!(self.increments >> 32, 0);
                dest.write_u32_le(self.increments as u32).await?;
            }
            5 => {
                assert_eq!(self.increments >> 40, 0);
                dest.write_u32_le(self.increments as u32).await?;
                dest.write_u8((self.increments >> 32) as u8).await?;
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// A presentation of `datetime2` type in the server.
pub struct DateTime2 {
    date: Date,
    time: Time,
}

impl DateTime2 {
    /// Construct a new `DateTime2` from the date and time components.
    pub fn new(date: Date, time: Time) -> Self {
        Self { date, time }
    }

    /// The date component.
    pub fn date(self) -> Date {
        self.date
    }

    /// The time component.
    pub fn time(self) -> Time {
        self.time
    }

    pub(crate) async fn decode<R>(src: &mut R, n: usize, rlen: usize) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let time = Time::decode(src, n, rlen as usize).await?;

        let mut bytes = [0u8; 4];
        src.read_exact(&mut bytes[..3]).await?;
        let date = Date::new(LittleEndian::read_u32(&bytes));

        Ok(Self::new(date, time))
    }

    pub(crate) async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.time.encode(dest).await?;

        let mut tmp = [0u8; 4];
        LittleEndian::write_u32(&mut tmp, self.date.days());
        assert_eq!(tmp[3], 0);
        dest.write_all(&tmp[0..3]).await?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// A presentation of `datetimeoffset` type in the server.
/// type with the correct timezone.
pub struct DateTimeOffset {
    datetime2: DateTime2,
    offset: i16,
}

impl DateTimeOffset {
    /// Construct a new `DateTimeOffset` from a `datetime2`, offset marking
    /// number of minutes from UTC.
    pub fn new(datetime2: DateTime2, offset: i16) -> Self {
        Self { datetime2, offset }
    }

    /// The date and time part.
    pub fn datetime2(self) -> DateTime2 {
        self.datetime2
    }

    /// Number of minutes from UTC.
    pub fn offset(self) -> i16 {
        self.offset
    }

    pub(crate) async fn decode<R>(src: &mut R, n: usize, rlen: u8) -> crate::Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let datetime2 = DateTime2::decode(src, n, rlen as usize).await?;
        let offset = src.read_i16_le().await?;

        Ok(Self { datetime2, offset })
    }

    pub(crate) async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.datetime2.encode(dest).await?;
        dest.write_i16_le(self.offset).await?;

        Ok(())
    }
}
