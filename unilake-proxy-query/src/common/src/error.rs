use crate::error_code::ErrorCode;
use std::char::DecodeUtf16Error;

pub type Result<T> = std::result::Result<T, ErrorCode>;

impl From<std::io::Error> for ErrorCode {
    fn from(value: std::io::Error) -> Self {
        todo!()
    }
}

impl From<DecodeUtf16Error> for ErrorCode {
    fn from(value: DecodeUtf16Error) -> Self {
        todo!()
    }
}
