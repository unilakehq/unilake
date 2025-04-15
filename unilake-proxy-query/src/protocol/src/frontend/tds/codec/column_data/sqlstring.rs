use crate::frontend::tds::codec::TypeInfo;
use tokio_util::bytes::BytesMut;
use unilake_common::error::Result;

#[derive(Debug, Clone)]
pub struct SqlString {
    max_length: usize,
    value: Option<String>,
}

impl SqlString {
    /// Creates a new `SqlString` instance from an optional string value and maximum length.
    /// `max_length` must equal the columns maximum length.
    ///
    /// # Parameters
    ///
    /// * `value` - An `Option<String>` representing the string value to be stored.
    ///             If `None`, the `SqlString` will be created without a value.
    /// * `max_length` - An `Option<usize>` specifying the maximum allowed length for the string.
    ///                  If `None`, it defaults to `usize::MAX`.
    ///
    /// # Returns
    ///
    /// Returns a new `SqlString` instance with the specified value and maximum length.
    pub fn from_string(value: Option<impl ToString>, max_length: Option<usize>) -> SqlString {
        let max_length = max_length.unwrap_or(usize::MAX);
        SqlString { max_length, value }
    }

    pub(crate) fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        if let Some(ref str) = self.value {
            super::plp::encode(dest, &self.max_length, Some(str));
        } else {
            super::plp::encode(dest, &self.max_length, None);
        }
        Ok(())
    }

    pub(crate) fn decode(src: &mut BytesMut, max_len: Option<usize>) -> Result<Self> {
        Ok(SqlString::from_string(
            super::plp::decode(src, &max_len.unwrap_or(usize::MAX))?,
            max_len,
        ))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    pub fn len(&self) -> usize {
        self.value.as_ref().map(|s| s.len()).unwrap_or(0)
    }

    pub fn new_empty(ty: &TypeInfo) -> SqlString {
        match ty {
            TypeInfo::VarLenSized(l) => SqlString {
                max_length: l.len(),
                value: None,
            },
            _ => unreachable!(),
        }
    }
}
