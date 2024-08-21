use crate::tds::codec::{decode, encode};
use crate::{
    Column, ColumnData, ColumnType, Error, FixedLenType, Result, TdsToken, TdsTokenCodec,
    TdsTokenType, TypeInfo, VarLenType,
};
use enumflags2::{bitflags, BitFlags};
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
    pub flags: BitFlags<ColumnFlag>,
    pub ty: TypeInfo,
}

#[bitflags]
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnFlag {
    /// The column can be null.
    Nullable = 1 << 0,
    /// Set for string columns with binary collation and always for the XML data
    /// type.
    CaseSensitive = 1 << 1,
    /// If column is writeable.
    // todo(mrhamburg): this flag is 2 bytes? messses up the other flags
    Updateable = 1 << 3,
    /// Column modification status unknown.
    UpdateableUnknown = 1 << 4,
    /// Column is an identity.
    Identity = 1 << 5,
    /// Column is computed.
    Computed = 1 << 7,
    /// Column is a fixed-length common language runtime user-defined type (CLR
    /// UDT).
    FixedLenClrType = 1 << 10,
    /// Column is the special XML column for the sparse column set.
    SparseColumnSet = 1 << 11,
    /// Column is encrypted transparently and has to be decrypted to view the
    /// plaintext value. This flag is valid when the column encryption feature
    /// is negotiated between client and server and is turned on.
    Encrypted = 1 << 12,
    /// Column is part of a hidden primary key created to support a T-SQL SELECT
    /// statement containing FOR BROWSE.
    Hidden = 1 << 13,
    /// Column is part of a primary key for the row and the T-SQL SELECT
    /// statement contains FOR BROWSE.
    Key = 1 << 14,
    /// It is unknown whether the column might be nullable.
    NullableUnknown = 1 << 15,
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
    pub fn new() -> Self {
        TokenColMetaData {
            columns: Vec::new(),
        }
    }

    /// Returns the number of columns in this metadata.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Adds a new column to the metadata. Returns the index of the added column.
    pub fn add_column(&mut self, col_name: &str, ty: TypeInfo) -> usize {
        self.columns.push(MetaDataColumn {
            base: BaseMetaDataColumn {
                flags: BitFlags::empty(),
                ty,
            },
            col_name: col_name.to_string(),
        });
        self.get_index(col_name).unwrap()
    }

    /// Returns the index of the column with the given name, or None if not found.
    pub fn get_index(&self, col_name: &str) -> Option<usize> {
        self.columns.iter().position(|x| x.col_name == col_name)
    }

    /// Sets the nullable flag for the column at the given index.
    pub fn column_set_nullable(&mut self, index: usize, is_nullable: bool) {
        self.columns[index]
            .base
            .flags
            .set(ColumnFlag::Nullable, is_nullable);
    }

    /// Sets the case sensitive flag for the column at the given index.
    pub fn column_set_case_sensitive(&mut self, index: usize, is_case_sensitive: bool) {
        self.columns[index]
            .base
            .flags
            .set(ColumnFlag::CaseSensitive, is_case_sensitive);
    }

    /// Sets the updateable flag for the column at the given index.
    pub fn column_set_updateable(&mut self, index: usize, is_updateable: bool) {
        self.columns[index]
            .base
            .flags
            .set(ColumnFlag::Updateable, is_updateable);
    }

    /// Sets the identity flag for the column at the given index.
    pub fn column_set_identity(&mut self, index: usize, is_identity: bool) {
        self.columns[index]
            .base
            .flags
            .set(ColumnFlag::Identity, is_identity);
    }

    /// Sets the computed flag for the column at the given index.
    pub fn column_set_computed(&mut self, index: usize, is_computed: bool) {
        self.columns[index]
            .base
            .flags
            .set(ColumnFlag::Computed, is_computed);
    }

    /// Sets the fixed length CLR type flag for the column at the given index.
    pub fn column_set_fixed_len_clr_type(&mut self, index: usize, is_fixed_len_clr_type: bool) {
        self.columns[index]
            .base
            .flags
            .set(ColumnFlag::FixedLenClrType, is_fixed_len_clr_type);
    }

    /// Sets the encrypted flag for the column at the given index.
    pub fn column_set_encrypted(&mut self, index: usize, is_encrypted: bool) {
        self.columns[index]
            .base
            .flags
            .set(ColumnFlag::Encrypted, is_encrypted);
    }

    /// Sets the hidden flag for the column at the given index.
    pub fn column_set_hidden(&mut self, index: usize, is_hidden: bool) {
        self.columns[index]
            .base
            .flags
            .set(ColumnFlag::Hidden, is_hidden);
    }

    /// Sets the key flag for the column at the given index.
    pub fn column_set_key(&mut self, index: usize, is_key: bool) {
        self.columns[index].base.flags.set(ColumnFlag::Key, is_key);
    }

    /// Sets the nullable unknown flag for the column at the given index.
    pub fn column_set_nullable_unknown(&mut self, index: usize, is_nullable_unknown: bool) {
        self.columns[index]
            .base
            .flags
            .set(ColumnFlag::NullableUnknown, is_nullable_unknown);
    }
}

impl TdsTokenCodec for TokenColMetaData {
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

    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::ColMetaData as u8);

        dest.put_u16_le(if self.columns.len() > 0 {
            self.columns.len() as u16
        } else {
            0xFFFF
        });

        for column in &self.columns {
            column.base.encode(dest)?;
            encode::write_b_varchar(dest, &column.col_name)?;
        }

        Ok(())
    }
}

impl BaseMetaDataColumn {
    fn null_value(&self) -> ColumnData {
        match self.ty {
            TypeInfo::FixedLen(ty) => match ty {
                FixedLenType::Null => ColumnData::I32(None),
                FixedLenType::Int1 => ColumnData::U8(None),
                FixedLenType::Bit => ColumnData::Bit(None),
                FixedLenType::Int2 => ColumnData::I16(None),
                FixedLenType::Int4 => ColumnData::I32(None),
                FixedLenType::Datetime4 => ColumnData::SmallDateTime(None),
                FixedLenType::Float4 => ColumnData::F32(None),
                FixedLenType::Datetime => ColumnData::DateTime(None),
                FixedLenType::Float8 => ColumnData::F64(None),
                FixedLenType::Int8 => ColumnData::I64(None),
            },
            TypeInfo::VarLenSized(cx) => match cx.r#type() {
                VarLenType::Intn => ColumnData::I32(None),
                VarLenType::Bitn => ColumnData::Bit(None),
                VarLenType::Decimaln => ColumnData::Numeric(None),
                VarLenType::Numericn => ColumnData::Numeric(None),
                VarLenType::Floatn => ColumnData::F32(None),
                VarLenType::Datetimen => ColumnData::DateTime(None),
                VarLenType::Daten => ColumnData::Date(None),
                VarLenType::Timen => ColumnData::Time(None),
                VarLenType::Datetime2 => ColumnData::DateTime2(None),
                VarLenType::DatetimeOffsetn => ColumnData::DateTimeOffset(None),
                VarLenType::BigVarBin => ColumnData::Binary(None),
                VarLenType::BigVarChar => ColumnData::String(None),
                VarLenType::BigBinary => ColumnData::Binary(None),
                VarLenType::BigChar => ColumnData::String(None),
                VarLenType::NVarchar => ColumnData::String(None),
                VarLenType::NChar => ColumnData::String(None),
                VarLenType::SSVariant => todo!(),
            },
            TypeInfo::VarLenSizedPrecision { ty, .. } => match ty {
                VarLenType::Intn => ColumnData::I32(None),
                VarLenType::Bitn => ColumnData::Bit(None),
                VarLenType::Decimaln => ColumnData::Numeric(None),
                VarLenType::Numericn => ColumnData::Numeric(None),
                VarLenType::Floatn => ColumnData::F32(None),
                VarLenType::Datetimen => ColumnData::DateTime(None),
                VarLenType::Daten => ColumnData::Date(None),
                VarLenType::Timen => ColumnData::Time(None),
                VarLenType::Datetime2 => ColumnData::DateTime2(None),
                VarLenType::DatetimeOffsetn => ColumnData::DateTimeOffset(None),
                VarLenType::BigVarBin => ColumnData::Binary(None),
                VarLenType::BigVarChar => ColumnData::String(None),
                VarLenType::BigBinary => ColumnData::Binary(None),
                VarLenType::BigChar => ColumnData::String(None),
                VarLenType::NVarchar => ColumnData::String(None),
                VarLenType::NChar => ColumnData::String(None),
                VarLenType::SSVariant => todo!(),
            },
        }
    }

    pub fn decode(src: &mut BytesMut) -> Result<Self> {
        let _user_ty = src.get_u32_le();

        let flags = BitFlags::from_bits(src.get_u16_le())
            .map_err(|_| Error::Protocol("column metadata: invalid flags".into()))?;

        let ty = TypeInfo::decode(src)?;

        Ok(BaseMetaDataColumn { flags, ty })
    }

    pub fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u32_le(0x00);
        dest.put_u16_le(self.flags.bits());
        self.ty.encode(dest)?;
        Ok(())
    }
}
