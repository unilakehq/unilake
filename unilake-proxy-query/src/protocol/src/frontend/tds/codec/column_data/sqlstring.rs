use crate::frontend::Result;
use tokio_util::bytes::BytesMut;

#[derive(Debug)]
pub struct SqlString {
    max_length: usize,
    value: Option<String>,
}

impl SqlString {
    pub fn from_str(value: &str, max_length: usize) -> SqlString {
        SqlString {
            max_length,
            value: Some(value.to_string()),
        }
    }

    pub(crate) fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        if let Some(ref str) = self.value {
            super::plp::encode(dest, &self.max_length, Some(str));
        } else {
            super::plp::encode(dest, &self.max_length, None);
        }
        Ok(())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.value.is_none()
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
