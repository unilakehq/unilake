use tokio_util::bytes::BytesMut;
use unilake_common::error::TdsWireResult;

#[derive(Debug, Clone)]
pub struct SqlString {
    max_length: usize,
    value: Option<String>,
}

impl SqlString {
    pub fn from_string(value: Option<String>, max_length: usize) -> SqlString {
        SqlString { max_length, value }
    }

    pub(crate) fn encode(&self, dest: &mut BytesMut) -> TdsWireResult<()> {
        if let Some(ref str) = self.value {
            super::plp::encode(dest, &self.max_length, Some(str));
        } else {
            super::plp::encode(dest, &self.max_length, None);
        }
        Ok(())
    }

    pub(crate) fn decode(src: &mut BytesMut, max_len: usize) -> TdsWireResult<Self> {
        Ok(SqlString::from_string(
            super::plp::decode(src, &max_len)?,
            max_len,
        ))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    pub fn len(&self) -> usize {
        self.value.as_ref().map(|s| s.len()).unwrap_or(0)
    }

    pub fn new_empty(ty: &crate::frontend::TypeInfo) -> SqlString {
        match ty {
            crate::frontend::TypeInfo::VarLenSized(l) => SqlString {
                max_length: l.len(),
                value: None,
            },
            _ => unreachable!(),
        }
    }
}
