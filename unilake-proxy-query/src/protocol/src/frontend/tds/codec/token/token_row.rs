use super::{TdsToken, TdsTokenCodec};
use crate::frontend::{ColumnData, TdsTokenType};
use tokio_util::bytes::{BufMut, BytesMut};
use unilake_common::error::TdsWireResult;

/// A row of data.
#[derive(Debug, Default)]
pub struct TokenRow {
    data: Vec<ColumnData>,
    pub nbc_row: bool,
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
            nbc_row,
        }
    }

    pub fn from<V>(value: V) -> Self
    where
        V: Into<Self>,
    {
        value.into()
    }

    /// Write/encode a nbc row to the client. Server decides whether to send an NBC row or a normal row.
    pub fn encode_nbc(&self, dest: &mut BytesMut) -> TdsWireResult<()> {
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

    /// The size of the row in bytes.
    pub fn size_in_bytes(&self) -> usize {
        let mut size = 0;
        for d in &self.data {
            size += d.size_in_bytes();
        }
        size
    }

    /// Gives the columnar data with the given index. `None` if index out of
    /// bounds.
    pub fn get(&self, index: usize) -> Option<&ColumnData> {
        self.data.get(index)
    }

    /// Adds a new value to the row.
    pub fn push<V>(&mut self, value: V)
    where
        V: Into<ColumnData>,
    {
        self.data.push(value.into());
    }

    pub fn push_row(&mut self, row: ColumnData) {
        self.data.push(row);
    }

    /// True if row has no columns.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl TdsTokenCodec for TokenRow {
    fn encode(&self, dest: &mut BytesMut) -> TdsWireResult<()> {
        if self.nbc_row {
            return self.encode_nbc(dest);
        }

        dest.put_u8(TdsTokenType::Row as u8);
        for (_, d) in self.data.iter().enumerate() {
            d.encode(dest)?;
        }

        Ok(())
    }

    /// Decode is not implemented for this token type.
    fn decode(_: &mut BytesMut) -> TdsWireResult<TdsToken> {
        unimplemented!()
    }
}

/// A bitmap of null values in the row. Sometimes the query result decides to pack the
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
        let mut data = Vec::with_capacity(size);
        data.resize(size, 0);
        Self { data }
    }

    fn from(source: &Vec<ColumnData>) -> Self {
        let len = source.len() as i32;
        let len = len / 8 + (len % 8).signum();
        let mut ret = Self::new(len as usize);

        for (i, d) in source.iter().enumerate() {
            match d {
                ColumnData::U8N(None)
                | ColumnData::BitN(None)
                | ColumnData::I16N(None)
                | ColumnData::I32N(None)
                | ColumnData::I64N(None)
                | ColumnData::F32N(None)
                | ColumnData::F64N(None)
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

        if index >= self.data.len() {
            self.data.resize(index + 1, 0);
        }

        self.data[index] |= 1 << bit;
    }

    /// Encode the bitmap data from the beginning of the row. Only doable if the
    /// type is `NbcRowToken`.
    fn encode(&self, dest: &mut BytesMut) -> TdsWireResult<()> {
        dest.extend_from_slice(&self.data);
        Ok(())
    }
}
