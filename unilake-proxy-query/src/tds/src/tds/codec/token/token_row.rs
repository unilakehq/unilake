use crate::{ColumnData, Result, TokenColMetaData, TokenType};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// A row of data.
#[derive(Debug, Default, Clone)]
pub struct TokenRow<'a> {
    data: Vec<ColumnData<'a>>,
}
impl<'a> IntoIterator for TokenRow<'a> {
    type Item = ColumnData<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> TokenRow<'a> {
    /// Creates a new empty row with allocated capacity from an existing token column metadata.
    pub fn from(col: &TokenColMetaData) -> Self {
        Self {
            data: Vec::with_capacity(col.columns.len()),
        }
    }

    /// Normal row. We'll read the metadata what we've cached and parse columns
    /// based on that.
    pub async fn decode<R>(src: &mut R, col_meta: Arc<TokenColMetaData>) -> Result<TokenRow<'a>>
    where
        R: AsyncRead + Unpin,
    {
        let mut row = Self {
            data: Vec::with_capacity(col_meta.columns.len()),
        };

        for column in col_meta.columns.iter() {
            let data = ColumnData::decode(src, &column.base.ty).await?;
            row.data.push(data);
        }

        Ok(row)
    }

    /// Write/encode a normal row to the client. Server decides whether to send an NBC row or a normal row.
    /// Since we do not support any variable length types, nbcrow should be used when a row has null values.
    pub async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TokenType::Row as u8).await?;

        for column in &self.data {
            column.encode(dest).await?;
        }

        Ok(())
    }

    /// SQL Server has packed nulls on this row type. We'll read what columns
    /// are null from the bitmap.
    pub async fn decode_nbc<R>(src: &mut R, col_meta: Arc<TokenColMetaData>) -> Result<TokenRow<'a>>
    where
        R: AsyncRead + Unpin,
    {
        let row_bitmap = RowBitmap::decode(src, col_meta.columns.len()).await?;

        let mut row = Self {
            data: Vec::with_capacity(col_meta.columns.len()),
        };

        for (i, column) in col_meta.columns.iter().enumerate() {
            let data = if row_bitmap.is_null(i) {
                column.base.null_value()
            } else {
                ColumnData::decode(src, &column.base.ty).await?
            };

            row.data.push(data);
        }

        Ok(row)
    }

    /// Write/encode an nbc row to the client. Server decides whether to send an NBC row or a normal row.
    pub async fn encode_nbc<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TokenType::NbcRow as u8).await?;
        let bm = RowBitmap::from(&self.data);
        bm.encode(dest).await?;
        for (i, d) in self.data.iter().enumerate() {
            if !bm.is_null(i) {
                d.encode(dest).await?;
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
    pub fn get(&self, index: usize) -> Option<&ColumnData<'a>> {
        self.data.get(index)
    }

    /// Adds a new value to the row.
    pub fn push(&mut self, value: ColumnData<'a>) {
        self.data.push(value);
    }

    /// True if row has no columns.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
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

    fn from(source: &Vec<ColumnData<'_>>) -> Self {
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
                | ColumnData::String(None)
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

    /// Decode the bitmap data from the beginning of the row. Only doable if the
    /// type is `NbcRowToken`.
    async fn decode<R>(src: &mut R, columns: usize) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let size = (columns + 8 - 1) / 8;
        let mut data = vec![0; size];
        src.read_exact(&mut data[0..size]).await?;

        Ok(Self { data })
    }

    /// Encode the bitmap data from the beginning of the row. Only doable if the
    /// type is `NbcRowToken`.
    async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_all(self.data.as_slice()).await?;
        Ok(())
    }
}
