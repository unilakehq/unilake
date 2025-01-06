//! Error module

use std::fmt;
pub use std::io::Error as IOError;
pub use std::io::ErrorKind as IoErrorKind;
use thiserror::Error;

/// Error token [2.2.7.10]
/// Used to send an error message to the client.
#[derive(Clone, Debug)]
pub struct TokenError {
    /// ErrorCode
    pub code: u32,
    /// ErrorState (describing code)
    pub state: u8,
    /// The class (severity) of the error
    pub class: u8,
    /// The error message
    pub message: String,
    pub server: String,
    pub procedure: String,
    pub line: u32,
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' on server {} executing {} on line {} (code: {}, state: {}, class: {})",
            self.message, self.server, self.procedure, self.line, self.code, self.state, self.class
        )
    }
}

impl TokenError {
    pub fn new(
        code: u32,
        state: u8,
        class: u8,
        message: String,
        server: String,
        procedure: String,
        line: u32,
    ) -> TokenError {
        TokenError {
            code,
            state,
            class,
            message,
            server,
            procedure,
            line,
        }
    }
}

/// A unified error enum that contains several errors that might occur during
/// the lifecycle of this codec
#[derive(Error, Debug)]
pub enum TdsWireError {
    #[error("Protocol error: {}", _0)]
    /// An error happened during the request or response parsing.
    Protocol(String),
    #[error("Encoding error: {}", _0)]
    /// Server responded with encoding not supported.
    Encoding(String),
    #[error("Conversion error: {}", _0)]
    /// Conversion failure from one type to another.
    Conversion(String),
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
    #[error("Column input failure: {0}")]
    /// Invalid input
    Input(String),
}

impl From<TdsWireError> for std::io::Error {
    fn from(e: TdsWireError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }
}
impl From<std::io::Error> for TdsWireError {
    fn from(value: std::io::Error) -> Self {
        println!(" Error occurred: {}", value);
        todo!()
    }
}

pub type TdsWireResult<T> = Result<T, TdsWireError>;
pub type Error = TdsWireError;
