use std::future::Future;

use libipld::Cid;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    message::{descriptor::Filter, DecodeError, Message},
    util::EncodeError,
};

#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "s3")]
mod s3;
#[cfg(feature = "surrealdb")]
mod surrealdb;

#[cfg(feature = "mysql")]
pub use mysql::MySQL;
#[cfg(feature = "s3")]
pub use s3::S3;
#[cfg(feature = "surrealdb")]
pub use surrealdb::SurrealDB;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("No data found for CID")]
    NotFound,
    #[error("Failed to interact with backend: {0}")]
    BackendError(anyhow::Error),
}

pub trait DataStore: Send + Sync {
    fn delete(&self, cid: String)
        -> impl Future<Output = Result<(), DataStoreError>> + Send + Sync;
    fn get(
        &self,
        cid: String,
    ) -> impl Future<Output = Result<Option<GetDataResults>, DataStoreError>> + Send + Sync;
    fn put(
        &self,
        cid: String,
        value: Vec<u8>,
    ) -> impl Future<Output = Result<PutDataResults, DataStoreError>> + Send + Sync;
}

#[derive(Debug)]
pub struct GetDataResults {
    pub size: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PutDataResults {
    #[serde(rename = "dataSize")]
    size: usize,
}

#[derive(Error, Debug)]
pub enum MessageStoreError {
    #[error("Message missing data")]
    MissingData,
    #[error(transparent)]
    MessageEncode(#[from] EncodeError),
    #[error(transparent)]
    DataEncodeError(#[from] libipld_core::error::SerdeError),
    #[error("Not found")]
    NotFound,
    #[error(transparent)]
    MessageDecodeError(#[from] DecodeError),
    #[error(transparent)]
    Cid(#[from] libipld::cid::Error),
    #[error("Failed to create block {0}")]
    CreateBlockError(anyhow::Error),
    #[error("Failed to interact with backend: {0}")]
    BackendError(anyhow::Error),
    #[error(transparent)]
    DataStoreError(#[from] DataStoreError),
}

pub trait MessageStore: Send + Sync {
    fn delete(
        &self,
        tenant: &str,
        cid: String,
        data_store: &impl DataStore,
    ) -> impl Future<Output = Result<(), MessageStoreError>> + Send + Sync;
    fn put(
        &self,
        tenant: String,
        message: Message,
        data_store: &impl DataStore,
    ) -> impl Future<Output = Result<Cid, MessageStoreError>> + Send + Sync;
    fn query(
        &self,
        tenant: &str,
        filter: Filter,
    ) -> impl Future<Output = Result<Vec<Message>, MessageStoreError>> + Send + Sync;
}
