pub(crate) static ABAC_MODEL: &str = include_str!("../abac_model.conf");

mod adapter;
mod caching;
pub mod context;
mod effector;
mod functions;
pub mod handler;
mod policies;
mod repository;
mod scanner;
