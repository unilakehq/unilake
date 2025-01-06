extern crate core;

pub(crate) use crate::frontend::tds::codec::*;

#[macro_use]
mod macros;

pub mod codec;
pub mod prot;
pub mod tds;
pub mod utils;

pub(crate) fn get_driver_version() -> u64 {
    env!("CARGO_PKG_VERSION")
        .splitn(6, '.')
        .enumerate()
        .fold(0u64, |acc, part| match part.1.parse::<u64>() {
            Ok(num) => acc | num << (part.0 * 8),
            _ => acc | 0 << (part.0 * 8),
        })
}
