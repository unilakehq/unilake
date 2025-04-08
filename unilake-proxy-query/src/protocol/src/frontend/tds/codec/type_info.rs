use crate::frontend::tds::collation::Collation;
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use unilake_common::error::{Error, TdsWireResult};

#[derive(Debug)]
pub enum TypeInfo {
    FixedLen(FixedLenType),
    VarLenSized(VarLenContext),
    VarLenSizedPrecision {
        ty: VarLenType,
        size: usize,
        precision: u8,
        scale: u8,
    },
}

impl TypeInfo {
    pub fn new_bit() -> Self {
        Self::FixedLen(FixedLenType::Bit)
    }
    pub fn new_tiny_intn(is_nullable: bool) -> Self {
        if is_nullable {
            return Self::VarLenSized(VarLenContext::new(VarLenType::Intn, 1, None));
        }
        Self::FixedLen(FixedLenType::Int1)
    }
    pub fn new_small_intn(is_nullable: bool) -> Self {
        if is_nullable {
            return Self::VarLenSized(VarLenContext::new(VarLenType::Intn, 2, None));
        }
        Self::FixedLen(FixedLenType::Int2)
    }
    pub fn new_intn(is_nullable: bool) -> Self {
        if is_nullable {
            return Self::VarLenSized(VarLenContext::new(VarLenType::Intn, 4, None));
        }
        Self::FixedLen(FixedLenType::Int4)
    }
    pub fn new_big_intn(is_nullable: bool) -> Self {
        if is_nullable {
            return Self::VarLenSized(VarLenContext::new(VarLenType::Intn, 8, None));
        }
        Self::FixedLen(FixedLenType::Int8)
    }
    pub fn new_decimaln(precision: u8, scale: u8) -> Self {
        Self::VarLenSizedPrecision {
            ty: VarLenType::Decimaln,
            size: match precision {
                1..=8 => 4,
                9..=18 => 8,
                19..=27 => 12,
                28..=38 => 16,
                _ => todo!("return unsupported precision error"),
            },
            precision,
            scale,
        }
    }
    pub fn new_floatn_32(is_nullable: bool) -> Self {
        if is_nullable {
            return Self::VarLenSized(VarLenContext::new(VarLenType::Floatn, 4, None));
        }
        Self::FixedLen(FixedLenType::Float4)
    }
    pub fn new_floatn_64(is_nullable: bool) -> Self {
        if is_nullable {
            return Self::VarLenSized(VarLenContext::new(VarLenType::Floatn, 8, None));
        }
        Self::FixedLen(FixedLenType::Float8)
    }
    pub fn new_daten(is_nullable: bool) -> Self {
        Self::VarLenSized(VarLenContext::new(
            VarLenType::Daten,
            if is_nullable { 0 } else { 3 },
            None,
        ))
    }
    pub fn new_datetime2() -> Self {
        Self::VarLenSized(VarLenContext::new(VarLenType::Datetime2, 7, None))
    }
    /// Creates a new `TypeInfo` instance for an NVARCHAR type.
    ///
    /// # Parameters
    ///
    /// * `max_len`: An `Option<usize>` that specifies the maximum length of the NVARCHAR.
    ///   - If `Some(value)`, `value` is used as the maximum length.
    ///   - If `None`, the maximum length is set to 0xFFFF (65535), which represents NVARCHAR(MAX).
    ///
    /// # Returns
    ///
    /// Returns a `TypeInfo` instance configured for an NVARCHAR type with the specified or default maximum length
    /// and default collation settings.
    pub fn new_nvarchar(max_len: Option<usize>) -> Self {
        Self::VarLenSized(VarLenContext::new(
            VarLenType::NVarchar,
            max_len.unwrap_or_else(|| 0xFFFF),
            Some(Collation::default()),
        ))
    }
    /// Creates a new `TypeInfo` instance for an SYSNAME type.
    /// this is a special type used in SQL Server for tables
    pub fn new_sysname() -> Self {
        Self::VarLenSized(VarLenContext::new(
            VarLenType::NVarchar,
            128,
            Some(Collation::default()),
        ))
    }
}

#[derive(Clone, Debug, Copy)]
pub struct VarLenContext {
    r#type: VarLenType,
    len: usize,
    collation: Option<Collation>,
}

impl VarLenContext {
    pub fn new(r#type: VarLenType, len: usize, collation: Option<Collation>) -> Self {
        Self {
            r#type,
            len,
            collation,
        }
    }

    pub fn new_nvarchar() -> Self {
        todo!()
    }

    /// Get the var len context's r#type.
    pub fn r#type(&self) -> VarLenType {
        self.r#type
    }

    /// Get the var len context's len.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get the var len context's collation.
    pub fn collation(&self) -> Option<Collation> {
        self.collation
    }
}

uint_enum! {
    /// 2.2.5.4.2
    #[repr(u8)]
    pub enum FixedLenType {
        Null = 0x1F,
        Int1 = 0x30,
        Bit = 0x32,
        Int2 = 0x34,
        Int4 = 0x38,
        Datetime4 = 0x3A,
        Float4 = 0x3B,
        Datetime = 0x3D,
        Float8 = 0x3E,
        Int8 = 0x7F,
    }
}

uint_enum! {
    /// 2.2.5.4.3
    #[repr(u8)]
    pub enum VarLenType {
        Intn = 0x26,
        Bitn = 0x68,
        Decimaln = 0x6A,
        Numericn = 0x6C,
        Floatn = 0x6D,
        Datetimen = 0x6F,
        Daten = 0x28,
        Timen = 0x29,
        Datetime2 = 0x2A,
        DatetimeOffsetn = 0x2B,
        BigVarBin = 0xA5,
        BigVarChar = 0xA7,
        BigBinary = 0xAD,
        BigChar = 0xAF,
        NVarchar = 0xE7,
        NChar = 0xEF,
    }
}

impl TypeInfo {
    pub fn decode(src: &mut BytesMut) -> TdsWireResult<Self> {
        let ty = src.get_u8();

        if let Ok(ty) = FixedLenType::try_from(ty) {
            return Ok(TypeInfo::FixedLen(ty));
        }

        match VarLenType::try_from(ty) {
            Err(()) => Err(Error::Protocol(
                format!("invalid or unsupported column type: {:?}", ty).into(),
            )),
            Ok(ty) => {
                let len = match ty {
                    VarLenType::Timen | VarLenType::DatetimeOffsetn | VarLenType::Datetime2 => {
                        src.get_u8() as usize
                    }
                    VarLenType::Daten => 3,
                    VarLenType::Bitn
                    | VarLenType::Intn
                    | VarLenType::Floatn
                    | VarLenType::Decimaln
                    | VarLenType::Numericn
                    | VarLenType::Datetimen => src.get_u8() as usize,
                    VarLenType::NChar
                    | VarLenType::BigChar
                    | VarLenType::NVarchar
                    | VarLenType::BigVarChar
                    | VarLenType::BigBinary
                    | VarLenType::BigVarBin => src.get_u16_le() as usize,
                };

                let collation = match ty {
                    VarLenType::BigChar
                    | VarLenType::NChar
                    | VarLenType::NVarchar
                    | VarLenType::BigVarChar => {
                        let codepage = src.get_u16_le();
                        let flags = src.get_u16_le();
                        let charset_id = src.get_u8();

                        Some(Collation::new(codepage, flags, charset_id))
                    }
                    _ => None,
                };

                let vty = match ty {
                    VarLenType::Decimaln | VarLenType::Numericn => {
                        let precision = src.get_u8();
                        let scale = src.get_u8();

                        TypeInfo::VarLenSizedPrecision {
                            size: len,
                            ty,
                            precision,
                            scale,
                        }
                    }
                    _ => {
                        let cx = VarLenContext::new(ty, len, collation);
                        TypeInfo::VarLenSized(cx)
                    }
                };

                Ok(vty)
            }
        }
    }

    pub fn encode(&self, dest: &mut BytesMut) {
        match self {
            TypeInfo::VarLenSized(ty) => {
                dest.put_u8(ty.r#type as u8);

                // write length (type_varlen)
                match ty.r#type {
                    VarLenType::Timen
                    | VarLenType::Datetime2
                    | VarLenType::Bitn
                    | VarLenType::Intn
                    | VarLenType::Floatn
                    | VarLenType::Datetimen => dest.put_u8(ty.len() as u8),
                    VarLenType::NChar
                    | VarLenType::BigChar
                    | VarLenType::NVarchar
                    | VarLenType::BigVarChar => dest.put_u16_le(ty.len() as u16),
                    VarLenType::Daten => {}
                    _ => unimplemented!("not yet implemented for {:?}", ty),
                }

                // write collation
                if let Some(c) = ty.collation {
                    dest.put_u16_le(c.codepage);
                    dest.put_u16_le(c.flags);
                    dest.put_u8(c.charset_id);
                }
            }
            TypeInfo::VarLenSizedPrecision {
                ty,
                size,
                precision,
                scale,
            } => match ty {
                VarLenType::Decimaln | VarLenType::Numericn => {
                    dest.put_u8(*ty as u8);
                    dest.put_u8(*size as u8);
                    dest.put_u8(*precision);
                    dest.put_u8(*scale);
                }
                _ => {}
            },
            TypeInfo::FixedLen(ty) => {
                dest.put_u8(*ty as u8);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::bytes::BytesMut;

    use super::TypeInfo;

    const RAW_BYTES_VARCHAR: &[u8] = &[0xa7, 0x0b, 0x00, 0x09, 0x04, 0xd0, 0x00, 0x34];

    #[test]
    fn decode_encode_roundtrip_varchar() {
        let mut buf = BytesMut::from(RAW_BYTES_VARCHAR);
        let decoded = TypeInfo::decode(&mut buf).unwrap();
        decoded.encode(&mut buf);

        assert_eq!(buf, BytesMut::from(RAW_BYTES_VARCHAR));
    }
}
