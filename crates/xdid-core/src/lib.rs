//! Core types for DID methods to implement.

use std::{future::Future, pin::Pin};

use did::Did;
use thiserror::Error;

pub mod did;
pub mod did_url;
pub mod document;
mod uri;

pub trait Method: Send + Sync {
    fn method_name(&self) -> &'static str;

    /// Attempt to resolve the provided DID to its DID document.
    fn resolve(
        &self,
        did: Did,
    ) -> Pin<Box<dyn Future<Output = Result<document::Document, ResolutionError>> + Send + Sync>>;
}

#[derive(Error, Debug)]
pub enum ResolutionError {
    #[error("invalid DID")]
    InvalidDid,
    #[error("resolution failed: {0}")]
    ResolutionFailed(String),
    #[error("unsupported method")]
    UnsupportedMethod,
}
