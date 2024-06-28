extern crate core;

pub(crate) use crate::tds::{codec::*, collation::*, numeric::*, time::*};
pub(crate) use error::Error;

#[macro_use]
mod macros;

pub mod error;
pub mod prot;
pub mod tds;
mod to_sql;

/// An alias for a result that holds crate's error type as the error.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn get_driver_version() -> u64 {
    env!("CARGO_PKG_VERSION")
        .splitn(6, '.')
        .enumerate()
        .fold(0u64, |acc, part| match part.1.parse::<u64>() {
            Ok(num) => acc | num << (part.0 * 8),
            _ => acc | 0 << (part.0 * 8),
        })
}
