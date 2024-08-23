use crate::frontend::tds::collation::Collation;
use crate::frontend::{Error, Result};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

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
    pub fn new_tinyint() -> Self {
        Self::FixedLen(FixedLenType::Int1)
    }
    pub fn new_smallint() -> Self {
        Self::FixedLen(FixedLenType::Int2)
    }
    pub fn new_int() -> Self {
        Self::FixedLen(FixedLenType::Int4)
    }
    pub fn new_bigint() -> Self {
        Self::FixedLen(FixedLenType::Int8)
    }
    pub fn new_decimal(precision: u8, scale: u8) -> Self {
        Self::VarLenSizedPrecision {
            ty: VarLenType::Decimaln,
            size: (precision + scale) as usize,
            precision: precision,
            scale: scale,
        }
    }
    pub fn new_float_32() -> Self {
        Self::FixedLen(FixedLenType::Float4)
    }
    pub fn new_float_64() -> Self {
        Self::FixedLen(FixedLenType::Float8)
    }
    pub fn new_date() -> Self {
        Self::VarLenSized(VarLenContext::new(VarLenType::Daten, 0, None))
    }
    pub fn new_datetime() -> Self {
        Self::VarLenSized(VarLenContext::new(VarLenType::Datetimen, 0, None))
    }
    pub fn new_nvarchar(max_len: usize) -> Self {
        Self::VarLenSized(VarLenContext::new(
            VarLenType::NVarchar,
            max_len,
            Some(Collation::default()),
        ))
    }
    pub fn new_string() -> Self {
        Self::VarLenSized(VarLenContext::new(
            VarLenType::NVarchar,
            0xFFFF,
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
    /// 2.2.5.4.2
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
        // TODO: this needs to be implemented
        SSVariant = 0x62,
    }
}

impl TypeInfo {
    pub fn decode(src: &mut BytesMut) -> Result<Self> {
        let ty = src.get_u8();

        if let Ok(ty) = FixedLenType::try_from(ty) {
            return Ok(TypeInfo::FixedLen(ty));
        }

        match VarLenType::try_from(ty) {
            Err(()) => {
                return Err(Error::Protocol(
                    format!("invalid or unsupported column type: {:?}", ty).into(),
                ))
            }
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
                    _ => todo!("not yet implemented for {:?}", ty),
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

    pub fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        match self {
            TypeInfo::VarLenSized(ty) => {
                dest.put_u8(ty.r#type as u8);

                // write length
                match ty.r#type {
                    VarLenType::Timen
                    | VarLenType::DatetimeOffsetn
                    | VarLenType::Datetime2
                    | VarLenType::Bitn
                    | VarLenType::Intn
                    | VarLenType::Floatn
                    | VarLenType::Datetimen => dest.put_u8(ty.len() as u8),
                    VarLenType::NChar
                    | VarLenType::BigChar
                    | VarLenType::NVarchar
                    | VarLenType::BigVarChar
                    | VarLenType::BigBinary
                    | VarLenType::BigVarBin => dest.put_u16_le(ty.len() as u16),
                    VarLenType::Daten => {
                        dest.put_u8(3 as u8);
                    }
                    _ => {}
                }

                // write collation
                match ty.collation {
                    Some(c) => {
                        dest.put_u16_le(c.codepage);
                        dest.put_u16_le(c.flags);
                        dest.put_u8(c.charset_id);
                    }
                    _ => {}
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
                    dest.put_u8(*precision as u8);
                    dest.put_u8(*scale as u8);
                }
                _ => {}
            },
            TypeInfo::FixedLen(ty) => {
                dest.put_u8(*ty as u8);
            }
        }

        Ok(())
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
        decoded.encode(&mut buf).unwrap();

        assert_eq!(buf, BytesMut::from(RAW_BYTES_VARCHAR));
    }
}
