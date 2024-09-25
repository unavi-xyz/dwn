use std::future::Future;

use base64::DecodeError;
use libipld::Cid;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    encode::EncodeError,
    message::{
        descriptor::{protocols::ProtocolsFilter, records::RecordsFilter},
        Message,
    },
};

#[cfg(feature = "s3")]
mod s3;
mod stored_data;
#[cfg(feature = "surrealdb")]
mod surrealdb;

#[cfg(feature = "s3")]
pub use s3::S3;
#[cfg(feature = "surrealdb")]
pub use surrealdb::SurrealStore;

use self::stored_data::StoredData;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("No data found for CID")]
    NotFound,
    #[error("Failed to interact with backend: {0}")]
    BackendError(anyhow::Error),
}

pub trait DataStore: Send + Sync {
    fn delete(&self, cid: &str) -> impl Future<Output = Result<(), DataStoreError>> + Send + Sync;

    fn get(
        &self,
        cid: &str,
    ) -> impl Future<Output = Result<Option<StoredData>, DataStoreError>> + Send + Sync;

    fn put(
        &self,
        cid: String,
        value: StoredData,
    ) -> impl Future<Output = Result<PutDataResults, DataStoreError>> + Send + Sync;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PutDataResults {
    #[serde(rename = "dataSize")]
    size: usize,
}

pub trait MessageStore: Send + Sync {
    fn delete(
        &self,
        tenant: &str,
        cid: &str,
        data_store: &impl DataStore,
    ) -> impl Future<Output = Result<(), MessageStoreError>> + Send + Sync;

    fn put(
        &self,
        authorized: bool,
        tenant: String,
        message: Message,
        data_store: &impl DataStore,
    ) -> impl Future<Output = Result<Cid, MessageStoreError>> + Send + Sync;

    fn query_protocols(
        &self,
        tenant: String,
        authorized: bool,
        filter: ProtocolsFilter,
    ) -> impl Future<Output = Result<Vec<Message>, MessageStoreError>> + Send + Sync;

    fn query_records(
        &self,
        tenant: String,
        author: Option<String>,
        authorized: bool,
        filter: RecordsFilter,
    ) -> impl Future<Output = Result<Vec<Message>, MessageStoreError>> + Send + Sync;
}

#[derive(Error, Debug)]
pub enum MessageStoreError {
    #[error("Message missing data")]
    MissingData,
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    DataEncode(#[from] libipld_core::error::SerdeError),
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error(transparent)]
    Cid(#[from] libipld::cid::Error),
    #[error("Failed to create block {0}")]
    CreateBlock(anyhow::Error),
    #[error("Failed to interact with backend: {0}")]
    Backend(anyhow::Error),
    #[error(transparent)]
    DataStore(#[from] DataStoreError),
}
