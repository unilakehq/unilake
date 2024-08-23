use crate::frontend::{ColumnData, Result, TdsTokenType};
use tokio_util::bytes::{BufMut, BytesMut};

use super::{TdsToken, TdsTokenCodec};

/// A row of data.
#[derive(Debug, Default)]
pub struct TokenRow {
    data: Vec<ColumnData>,
    nbc_row: bool,
}
impl IntoIterator for TokenRow {
    type Item = ColumnData;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl TokenRow {
    /// Creates a new empty row with allocated capacity from an existing token column metadata.
    pub fn new(col_len: usize, nbc_row: bool) -> Self {
        Self {
            data: Vec::with_capacity(col_len),
            nbc_row: nbc_row,
        }
    }

    /// Write/encode an nbc row to the client. Server decides whether to send an NBC row or a normal row.
    pub fn encode_nbc(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::NbcRow as u8);
        let bm = RowBitmap::from(&self.data);
        bm.encode(dest)?;
        for (i, d) in self.data.iter().enumerate() {
            if !bm.is_null(i) {
                d.encode(dest)?;
            }
        }

        Ok(())
    }

    /// The number of columns.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Gives the columnar data with the given index. `None` if index out of
    /// bounds.
    pub fn get(&self, index: usize) -> Option<&ColumnData> {
        self.data.get(index)
    }

    /// Adds a new value to the row.
    pub fn push(&mut self, value: ColumnData) {
        self.data.push(value);
    }

    /// True if row has no columns.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl TdsTokenCodec for TokenRow {
    /// Decode is not implemented for this token type.
    fn decode(_: &mut BytesMut) -> Result<TdsToken> {
        unimplemented!()
    }

    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        if self.nbc_row {
            return self.encode_nbc(dest);
        }

        dest.put_u8(TdsTokenType::Row as u8);
        for (_, d) in self.data.iter().enumerate() {
            d.encode(dest)?;
        }

        Ok(())
    }
}

/// A bitmap of null values in the row. Sometimes SQL Server decides to pack the
/// null values in the row, calling it the NBCROW. In this kind of tokens the row
/// itself skips the null columns completely, but they can be found from the bitmap
/// stored in the beginning of the token.
///
/// One byte can store eight bits of information. Bits with value of one being null.
///
/// If our row has eight columns, and our byte in bits is:
///
/// ```ignore
/// 1 0 0 1 0 1 0 0
/// ```
///
/// This would mean columns 0, 3 and 5 are null and should not be parsed at all.
/// For more than eight columns, more bits need to be reserved for the bitmap
/// (see the size calculation).
struct RowBitmap {
    data: Vec<u8>,
}

impl RowBitmap {
    fn new(size: usize) -> Self {
        Self {
            data: Vec::with_capacity(size),
        }
    }

    fn from(source: &Vec<ColumnData>) -> Self {
        let mut ret = Self::new(std::cmp::min(source.len() / 8, 1));

        for (i, d) in source.iter().enumerate() {
            match d {
                ColumnData::U8(None)
                | ColumnData::I16(None)
                | ColumnData::I32(None)
                | ColumnData::I64(None)
                | ColumnData::F32(None)
                | ColumnData::F64(None)
                | ColumnData::Bit(None)
                | ColumnData::Binary(None)
                | ColumnData::Numeric(None)
                | ColumnData::DateTime(None)
                | ColumnData::SmallDateTime(None)
                | ColumnData::Time(None)
                | ColumnData::Date(None)
                | ColumnData::DateTime2(None)
                | ColumnData::DateTimeOffset(None) => {
                    ret.set_null(i);
                }
                ColumnData::String(s) => {
                    // If the string is empty, we assume it is null
                    if s.is_empty() {
                        ret.set_null(i);
                    }
                }
                _ => {}
            }
        }

        ret
    }

    /// Is the given column index null or not.
    #[inline]
    fn is_null(&self, i: usize) -> bool {
        let index = i / 8;
        let bit = i % 8;

        self.data[index] & (1 << bit) > 0
    }

    /// Set the given column index as null.
    #[inline]
    fn set_null(&mut self, i: usize) {
        let index = i / 8;
        let bit = i % 8;

        if (index + 1) > self.data.len() {
            self.data.resize(self.data.len() + 1, 0);
        }

        self.data[index] |= 1 << bit;
    }

    /// Encode the bitmap data from the beginning of the row. Only doable if the
    /// type is `NbcRowToken`.
    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.extend_from_slice(&self.data);
        Ok(())
    }
}
