use thiserror::Error;

mod data;
mod record;

pub use data::*;
pub use record::*;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("backend error: {0}")]
    BackendError(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
