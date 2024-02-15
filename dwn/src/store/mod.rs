use std::future::Future;

use libipld::Cid;
use serde::{Deserialize, Serialize};

use crate::message::Message;

#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "s3")]
pub mod s3;
#[cfg(feature = "surrealdb")]
pub mod surrealdb;

#[derive(thiserror::Error, Debug)]
pub enum DataStoreError {}

pub trait DataStore {
    fn delete(&self, tenant: &str, record_id: String, cid: Cid) -> Result<(), DataStoreError>;
    fn get<T: std::io::Read + Send + Sync>(
        &self,
        tenant: &str,
        record_id: String,
        cid: Cid,
    ) -> Result<Option<GetDataResults<T>>, DataStoreError>;
    fn put(
        &self,
        tenant: &str,
        record_id: String,
        cid: Cid,
        value: impl std::io::Write + Send + Sync,
    ) -> Result<PutDataResults, DataStoreError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDataResults<T>
where
    T: std::io::Read + Send + Sync,
{
    #[serde(rename = "dataSize")]
    size: usize,
    #[serde(rename = "dataStream")]
    data: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PutDataResults {
    #[serde(rename = "dataSize")]
    size: usize,
}

pub trait MessageStore {
    type Error: std::error::Error + Send + Sync + 'static;

    fn delete(&self, tenant: &str, cid: String) -> Result<(), Self::Error>;
    fn get(&self, tenant: &str, cid: String) -> Result<Message, Self::Error>;
    fn put(&self, tenant: &str, message: Message)
        -> impl Future<Output = Result<Cid, Self::Error>>;
}
