use std::{future::Future, pin::Pin, sync::Arc};

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
    fn delete(
        &self,
        cid: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), DataStoreError>> + Send + Sync>>;

    fn get(
        &self,
        cid: String,
    ) -> Pin<Box<dyn Future<Output = Result<Option<StoredData>, DataStoreError>> + Send + Sync>>;

    fn put(
        &self,
        cid: String,
        data: StoredData,
    ) -> Pin<Box<dyn Future<Output = Result<PutDataResults, DataStoreError>> + Send + Sync>>;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PutDataResults {
    #[serde(rename = "dataSize")]
    size: usize,
}

pub trait MessageStore: Send + Sync {
    fn delete(
        &self,
        tenant: String,
        cid: String,
        data_store: Arc<dyn DataStore>,
    ) -> Pin<Box<dyn Future<Output = Result<(), MessageStoreError>> + Send + Sync>>;

    fn put(
        &self,
        authorized: bool,
        tenant: String,
        message: Message,
        data_store: Arc<dyn DataStore>,
    ) -> Pin<Box<dyn Future<Output = Result<Cid, MessageStoreError>> + Send + Sync>>;

    fn query_protocols(
        &self,
        tenant: String,
        authorized: bool,
        filter: ProtocolsFilter,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Message>, MessageStoreError>> + Send + Sync>>;

    fn query_records(
        &self,
        tenant: String,
        author: Option<String>,
        authorized: bool,
        filter: RecordsFilter,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Message>, MessageStoreError>> + Send + Sync>>;
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
