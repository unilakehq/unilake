//! Error module
pub use crate::tds::codec::TokenError;
pub use std::io::ErrorKind as IoErrorKind;
use std::io::ErrorKind;
use std::rc::Rc;
use std::{borrow::Cow, io};
use thiserror::Error;

/// A unified error enum that contains several errors that might occur during
/// the lifecycle of this codec
#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("An error occurred during the attempt of performing I/O: {}", message)]
    /// An error occurred when performing I/O to the server.
    Io {
        /// A list specifying general categories of I/O error.
        kind: IoErrorKind,
        /// The error description.
        message: String,
    },
    #[error("Protocol error: {}", _0)]
    /// An error happened during the request or response parsing.
    Protocol(Cow<'static, str>),
    #[error("Encoding error: {}", _0)]
    /// Server responded with encoding not supported.
    Encoding(Cow<'static, str>),
    #[error("Conversion error: {}", _0)]
    /// Conversion failure from one type to another.
    Conversion(Cow<'static, str>),
    #[error("UTF-8 error")]
    /// Tried to convert data to UTF-8 that was not valid.
    Utf8,
    #[error("UTF-16 error")]
    /// Tried to convert data to UTF-16 that was not valid.
    Utf16,
    #[error("Error parsing an integer: {}", _0)]
    /// Tried to parse an integer that was not an integer.
    ParseInt(std::num::ParseIntError),
    #[error("Token error: {}", _0)]
    /// An error returned by the server.
    Server(TokenError),
    #[error("Error forming TLS connection: {}", _0)]
    /// An error in the TLS handshake.
    Tls(String),
    #[error("Generic error: {}", _0)]
    /// Generic error
    Err(Rc<dyn std::error::Error>),
    #[error("Column input failure: {0}")]
    /// Invalid input
    Input(Cow<'static, str>),
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, msg: &str) -> Error {
        todo!()
    }
}

impl Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Err(Rc::new(e))
    }
}
