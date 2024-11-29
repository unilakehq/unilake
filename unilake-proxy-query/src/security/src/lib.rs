pub(crate) static ABAC_MODEL: &str = include_str!("../abac_model.conf");

mod adapter;
mod caching;
mod effector;
mod functions;
pub mod handler;
mod policies;
mod repository;
mod scanner;

// re-exports
pub use crate::policies::HitRule;
pub use casbin::Adapter;
pub use casbin::Cache;
pub use casbin::DefaultCache;
