use thiserror::Error;

/// Errors that can happen while using this crate.
#[derive(Error, Debug)]
pub enum FlattenError {}

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
}

#[derive(Error, Debug)]
pub enum SchemaError {}

#[derive(Error, Debug)]
pub enum GenericError {}

#[derive(Error, Debug)]
pub enum ParquetError {}
