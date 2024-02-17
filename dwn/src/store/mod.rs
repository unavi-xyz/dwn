use std::{error::Error, future::Future};

use libipld::Cid;
use serde::{Deserialize, Serialize};

use crate::message::Message;

#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "s3")]
pub mod s3;
#[cfg(feature = "surrealdb")]
pub mod surrealdb;

pub trait DataStore {
    type Error: Error + Send + Sync + 'static;

    fn delete(&self, tenant: &str, record_id: &str, cid: Cid) -> Result<(), Self::Error>;
    fn get<T: std::io::Read + Send + Sync>(
        &self,
        tenant: &str,
        record_id: &str,
        cid: Cid,
    ) -> Result<Option<GetDataResults<T>>, Self::Error>;
    fn put(
        &self,
        tenant: &str,
        record_id: &str,
        cid: Cid,
        value: impl std::io::Write + Send + Sync,
    ) -> Result<PutDataResults, Self::Error>;
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
    type Error: Error + Send + Sync + 'static;

    fn delete(&self, tenant: &str, cid: String) -> impl Future<Output = Result<(), Self::Error>>;
    fn get(&self, tenant: &str, cid: &str) -> impl Future<Output = Result<Message, Self::Error>>;
    fn put(&self, tenant: &str, message: Message)
        -> impl Future<Output = Result<Cid, Self::Error>>;
}
