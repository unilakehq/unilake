use crate::frontend::tds::codec::{decode, encode};
use crate::frontend::{
    Column, ColumnType, Result, TdsToken, TdsTokenCodec, TdsTokenType, TypeInfo,
};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

/// Column Metadata Token [2.2.7.4]
/// Describes the result set for interpretation of following ROW data streams.
#[derive(Debug)]
pub struct TokenColMetaData {
    pub columns: Vec<MetaDataColumn>,
}

#[derive(Debug)]
pub struct MetaDataColumn {
    pub base: BaseMetaDataColumn,
    pub col_name: String,
}

#[derive(Debug)]
pub struct BaseMetaDataColumn {
    pub flags: DataFlags,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpdatableFlags {
    #[default]
    NotUpdatable = 0,
    Updatable = 1,
    Unknown = 2,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DataFlags {
    pub is_case_sensitive: bool,
    pub is_nullable: bool,
    pub updatable: UpdatableFlags,
    pub is_identity: bool,
    pub is_computed: bool,
    pub reserved_odbc: u8,
    pub is_fixed_length_clr: bool,
    pub is_default: bool,
    pub is_sparse_column_set: bool,
    pub is_encrypted: bool,
    pub is_hidden: bool,
    pub is_key: bool,
    pub is_nullable_unknown: bool,
}

impl DataFlags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_flags(flags: u16) -> Self {
        Self {
            is_nullable: (flags & 0x1) != 0,
            is_case_sensitive: ((flags >> 1) & 0x1) != 0,
            updatable: match (flags >> 2) & 0x3 {
                0 => UpdatableFlags::NotUpdatable,
                1 => UpdatableFlags::Updatable,
                _ => UpdatableFlags::Unknown,
            },
            is_identity: ((flags >> 4) & 0x1) != 0,
            is_computed: ((flags >> 5) & 0x1) != 0,
            reserved_odbc: ((flags >> 6) & 0x3) as u8,
            is_fixed_length_clr: ((flags >> 8) & 0x1) != 0,
            is_default: ((flags >> 9) & 0x1) != 0,
            is_sparse_column_set: ((flags >> 10) & 0x1) != 0,
            is_encrypted: ((flags >> 12) & 0x1) != 0,
            is_hidden: ((flags >> 13) & 0x1) != 0,
            is_key: ((flags >> 14) & 0x1) != 0,
            is_nullable_unknown: ((flags >> 15) & 0x1) != 0,
        }
    }

    pub fn to_u16(&self) -> u16 {
        (if self.is_nullable { 0x1 } else { 0x0 })
            | (if self.is_case_sensitive { 0x1 } else { 0x0 }) << 1
            | (self.updatable as u16) << 2
            | (if self.is_identity { 0x1 } else { 0x0 }) << 4
            | (if self.is_computed { 0x1 } else { 0x0 }) << 5
            | (self.reserved_odbc as u16) << 6
            | (if self.is_fixed_length_clr { 0x1 } else { 0x0 }) << 8
            | (if self.is_default { 0x1 } else { 0x0 }) << 9
            | (if self.is_sparse_column_set { 0x1 } else { 0x0 }) << 10
            | (if self.is_encrypted { 0x1 } else { 0x0 }) << 12
            | (if self.is_hidden { 0x1 } else { 0x0 }) << 13
            | (if self.is_key { 0x1 } else { 0x0 }) << 14
            | (if self.is_nullable_unknown { 0x1 } else { 0x0 }) << 15
    }
}
impl TokenColMetaData {
    /// Returns an iterator over the columns in this metadata.
    pub fn columns(&self) -> impl Iterator<Item = Column> + '_ {
        self.columns.iter().map(|x| Column {
            name: x.col_name.clone(),
            column_type: ColumnType::from(&x.base.ty),
        })
    }

    /// Creates a new empty column metadata token.
    pub fn new(len: usize) -> Self {
        TokenColMetaData {
            columns: Vec::with_capacity(len),
        }
    }

    /// Returns the number of columns in this metadata.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Adds a new column to the metadata. Returns the index of the added column.
    pub fn add_column<F>(&mut self, item: F) -> usize
    where
        F: Into<MetaDataColumn>,
    {
        self.columns.push(item.into());
        self.columns.len() - 1
    }

    /// Returns the index of the column with the given name, or None if not found.
    pub fn get_index(&self, col_name: &str) -> Option<usize> {
        self.columns.iter().position(|x| x.col_name == col_name)
    }

    /// Sets the nullable flag for the column at the given index.
    pub fn column_set_nullable(&mut self, index: usize, is_nullable: bool) {
        self.columns[index].base.flags.is_nullable = is_nullable;
    }

    /// Sets the case-sensitive flag for the column at the given index.
    pub fn column_set_case_sensitive(&mut self, index: usize, is_case_sensitive: bool) {
        self.columns[index].base.flags.is_case_sensitive = is_case_sensitive;
    }

    /// Sets the updatable flag for the column at the given index.
    pub fn column_set_updatable(&mut self, index: usize, is_updatable: UpdatableFlags) {
        self.columns[index].base.flags.updatable = is_updatable;
    }

    /// Sets the identity flag for the column at the given index.
    pub fn column_set_identity(&mut self, index: usize, is_identity: bool) {
        self.columns[index].base.flags.is_identity = is_identity;
    }

    /// Sets the computed flag for the column at the given index.
    pub fn column_set_computed(&mut self, index: usize, is_computed: bool) {
        self.columns[index].base.flags.is_computed = is_computed;
    }

    /// Sets the fixed length CLR type flag for the column at the given index.
    pub fn column_set_fixed_len_clr_type(&mut self, index: usize, is_fixed_len_clr_type: bool) {
        self.columns[index].base.flags.is_fixed_length_clr = is_fixed_len_clr_type;
    }

    /// Sets the encrypted flag for the column at the given index.
    pub fn column_set_encrypted(&mut self, index: usize, is_encrypted: bool) {
        self.columns[index].base.flags.is_encrypted = is_encrypted;
    }

    /// Sets the hidden flag for the column at the given index.
    pub fn column_set_hidden(&mut self, index: usize, is_hidden: bool) {
        self.columns[index].base.flags.is_hidden = is_hidden;
    }

    /// Sets the key flag for the column at the given index.
    pub fn column_set_key(&mut self, index: usize, is_key: bool) {
        self.columns[index].base.flags.is_key = is_key;
    }
}

impl TdsTokenCodec for TokenColMetaData {
    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::ColMetaData as u8);

        // fixed length value if there are no columns' metadata
        dest.put_u16_le(if self.columns.len() > 0 {
            self.columns.len() as u16
        } else {
            0xFFFF
        });

        for column in &self.columns {
            // push column base metadata
            column.base.encode(dest)?;

            // push column name (length + value)
            encode::write_b_varchar(dest, &column.col_name)?;
        }

        Ok(())
    }

    fn decode(src: &mut BytesMut) -> Result<TdsToken> {
        let column_count = src.get_u16_le();
        let mut columns = Vec::with_capacity(column_count as usize);

        if column_count > 0 && column_count < 0xffff {
            for _ in 0..column_count {
                let base = BaseMetaDataColumn::decode(src)?;
                let col_name = decode::read_b_varchar(src)?;

                columns.push(MetaDataColumn { base, col_name });
            }
        }

        Ok(TdsToken::ColMetaData(TokenColMetaData { columns }))
    }
}

impl BaseMetaDataColumn {
    pub fn decode(src: &mut BytesMut) -> Result<Self> {
        let _user_ty = src.get_u32_le();
        let flags = DataFlags::from_flags(src.get_u16_le());
        let ty = TypeInfo::decode(src)?;

        Ok(BaseMetaDataColumn { flags, ty })
    }

    pub fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        // user type
        dest.put_u32_le(0x00);
        // flags
        dest.put_u16_le(self.flags.to_u16());
        // token data (type, length, etc.)
        self.ty.encode(dest)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::{DataFlags, Result, UpdatableFlags};
    use tokio_util::bytes::{BufMut, BytesMut};

    const RAW_DATA: &[u8] = &[0x11, 0x00];

    #[test]
    fn col_metadata_flags_test() -> Result<()> {
        let mut bytes = BytesMut::new();
        let mut flags = DataFlags::default();
        flags.is_identity = true;
        flags.is_nullable = true;
        flags.updatable = UpdatableFlags::NotUpdatable;
        println!("{:x?}", flags.to_u16());
        bytes.put_u16_le(flags.to_u16());

        println!("{:x?}", bytes.to_vec());
        assert_eq!(bytes.to_vec(), RAW_DATA.to_vec());
        Ok(())
    }
}
