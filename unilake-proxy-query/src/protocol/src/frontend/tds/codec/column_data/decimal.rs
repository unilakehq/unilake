use crate::frontend::TdsTokenCodec;
use bigdecimal::{num_bigint::Sign, BigDecimal, ToPrimitive};
use tokio_util::bytes::{BufMut, BytesMut};
use unilake_common::error::TdsWireResult;

/// Represent a sql Decimal type. It is stored in an i128 and has a
/// maximum precision of 38 decimals.
#[derive(Copy, Clone, Debug)]
pub struct Decimal {
    value: i128,
    scale: u8,
}

// todo(mrhamburg): implement serialization for mysql_async, instead of making use of BigDecimal.
// todo(mrhamburg): check this implementation as azdatastudio is having issues with decimal precision. Query failed: Invalid numeric precision/scale.
// the above error can be replicated using: select top 10 Amount from [FactFinance]
impl Decimal {
    /// Creates a new Decimal value.
    ///
    /// # Panic
    /// It will panic if the scale exceed 37.
    pub fn new_with_scale(value: i128, scale: u8) -> Self {
        // scale cannot exceed 37 since a
        // max precision of 38 is possible here.
        assert!(scale < 38);
        Decimal { value, scale }
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

    pub(crate) fn encode(&self, dst: &mut BytesMut) -> TdsWireResult<()> {
        dst.put_u8(self.len());

        if self.value < 0 {
            dst.put_u8(0);
        } else {
            dst.put_u8(1);
        }

        let value = self.value().abs();

        match self.len() {
            5 => dst.put_u32_le(value as u32),
            9 => dst.put_u64_le(value as u64),
            13 => {
                dst.put_u64_le(value as u64);
                dst.put_u32_le((value >> 64) as u32)
            }
            _ => dst.put_u128_le(value as u128),
        }

        Ok(())
    }
}

impl TdsTokenCodec for BigDecimal {
    fn encode(&self, dst: &mut BytesMut) -> TdsWireResult<()> {
        let value = self.abs() * 10i128.pow(self.fractional_digit_count() as u32);
        let value = value.to_i128().unwrap();

        fn len(item: &BigDecimal) -> u8 {
            match item.digits() {
                1..=9 => 5,
                10..=19 => 9,
                20..=28 => 13,
                _ => 17,
            }
        }

        dst.put_u8(len(self));

        if self.sign() == Sign::Minus {
            dst.put_u8(0);
        } else {
            dst.put_u8(1);
        }

        match len(self) {
            5 => dst.put_u32_le(value as u32),
            9 => dst.put_u64_le(value as u64),
            13 => {
                dst.put_u64_le(value as u64);
                dst.put_u32_le((value >> 64) as u32)
            }
            _ => dst.put_u128_le(value as u128),
        }

        Ok(())
    }

    fn decode(_: &mut BytesMut) -> TdsWireResult<crate::frontend::TdsToken> {
        unimplemented!()
    }
}
