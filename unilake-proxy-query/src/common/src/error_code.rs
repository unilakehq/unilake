#![allow(non_snake_case)]

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use thiserror::Error;

#[derive(Error)]
pub struct ErrorCode {
    pub(crate) code: u16,
    pub(crate) name: String,
    pub(crate) display_text: String,
    pub(crate) detail: String,
}

impl ErrorCode {
    pub fn code(&self) -> u16 {
        self.code
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn display_text(&self) -> String {
        self.display_text.clone()
    }

    pub fn message(&self) -> String {
        let msg = self.display_text();
        if self.detail.is_empty() {
            msg
        } else {
            format!("{}\n{}", msg, self.detail)
        }
    }

    pub fn detail(&self) -> String {
        self.detail.clone()
    }

    #[must_use]
    pub fn add_message(self, msg: impl AsRef<str>) -> Self {
        Self {
            display_text: if self.display_text.is_empty() {
                msg.as_ref().to_string()
            } else {
                format!("{}\n{}", msg.as_ref(), self.display_text)
            },
            ..self
        }
    }

    #[must_use]
    pub fn add_message_back(self, msg: impl AsRef<str>) -> Self {
        Self {
            display_text: if self.display_text.is_empty() {
                msg.as_ref().to_string()
            } else {
                format!("{}\n{}", self.display_text, msg.as_ref())
            },
            ..self
        }
    }

    pub fn add_detail_back(self, msg: impl AsRef<str>) -> Self {
        Self {
            detail: if self.detail.is_empty() {
                msg.as_ref().to_string()
            } else {
                format!("{}\n{}", self.detail, msg.as_ref())
            },
            ..self
        }
    }

    pub fn add_detail(self, msg: impl AsRef<str>) -> Self {
        Self {
            detail: if self.detail.is_empty() {
                msg.as_ref().to_string()
            } else {
                format!("{}\n{}", msg.as_ref(), self.detail)
            },
            ..self
        }
    }
}

impl Debug for ErrorCode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}. Code: {}, Text = {}.",
            self.name,
            self.code(),
            self.message(),
        )?;
        Ok(())
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}. Code: {}, Text = {}.",
            self.name,
            self.code(),
            self.message(),
        )
    }
}

impl ErrorCode {
    /// All std error will be converted to InternalError
    #[track_caller]
    pub fn from_std_error<T: std::error::Error>(error: T) -> Self {
        ErrorCode {
            code: 1001,
            name: String::from("FromStdError"),
            display_text: error.to_string(),
            detail: String::new(),
        }
    }

    pub fn from_string(error: String) -> Self {
        ErrorCode {
            code: 1001,
            name: String::from("Internal"),
            display_text: error.clone(),
            detail: String::new(),
        }
    }

    pub fn from_string_no_backtrace(error: String) -> Self {
        ErrorCode {
            code: 1001,
            name: String::from("Internal"),
            display_text: error,
            detail: String::new(),
        }
    }

    pub fn create(code: u16, name: impl ToString, display_text: String, detail: String) -> Self {
        ErrorCode {
            code,
            display_text: display_text.clone(),
            detail,
            name: name.to_string(),
        }
    }
}
