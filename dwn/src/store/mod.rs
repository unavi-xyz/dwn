use std::future::Future;

use libipld::Cid;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::message::{DecodeError, EncodeError, Message};

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
    #[error("Failed to write data: {0}")]
    WriteError(#[from] std::io::Error),
    #[error("No data found for CID")]
    NotFound,
    #[error("Failed to interact with backend: {0}")]
    BackendError(anyhow::Error),
}

pub trait DataStore {
    fn delete(&self, cid: &Cid) -> impl Future<Output = Result<(), DataStoreError>>;
    fn get(
        &self,
        cid: &Cid,
    ) -> impl Future<Output = Result<Option<GetDataResults>, DataStoreError>>;
    fn put(
        &self,
        cid: &Cid,
        value: Vec<u8>,
    ) -> impl Future<Output = Result<PutDataResults, DataStoreError>>;
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
    #[error("Failed to generate CID: {0}")]
    MessageEncode(#[from] EncodeError),
    #[error("Failed to encode data: {0}")]
    DataEncodeError(#[from] libipld_core::error::SerdeError),
    #[error("Not found")]
    NotFound,
    #[error("Failed to decode message: {0}")]
    MessageDecodeError(#[from] DecodeError),
    #[error("Failed to generate CID: {0}")]
    Cid(#[from] libipld::cid::Error),
    #[error("Failed to create block {0}")]
    CreateBlockError(anyhow::Error),
    #[error("Failed to interact with backend: {0}")]
    BackendError(anyhow::Error),
}

pub trait MessageStore {
    fn delete(
        &self,
        tenant: &str,
        cid: String,
    ) -> impl Future<Output = Result<(), MessageStoreError>>;
    fn get(
        &self,
        tenant: &str,
        cid: &str,
    ) -> impl Future<Output = Result<Message, MessageStoreError>>;
    fn put(
        &self,
        tenant: &str,
        message: &Message,
    ) -> impl Future<Output = Result<Cid, MessageStoreError>>;
}

#[cfg(test)]
mod tests {
    // Generic tests for all data stores
    // Should be added to each data store's test suite
    pub mod data {
        use super::super::*;

        pub async fn test_all_methods(store: impl DataStore) {
            let cid = Cid::default();
            let data = vec![1, 2, 3, 4, 5];

            // Test put and get
            store
                .put(&cid, data.clone())
                .await
                .expect("Failed to put data");

            let got = store
                .get(&cid)
                .await
                .expect("Failed to get data")
                .expect("No data found");

            assert_eq!(data, got.data);

            // Test delete
            store.delete(&cid).await.expect("Failed to delete data");

            let got = store.get(&cid).await;

            assert!(got.is_ok());
            assert!(got.unwrap().is_none());
        }
    }

    // Generic tests for all message stores
    // Should be added to each message store's test suite
    pub mod message {
        use super::super::*;
        use crate::message::{
            descriptor::{Descriptor, RecordsWrite},
            Data, Message,
        };

        pub async fn test_all_methods(store: impl MessageStore) {
            let data = Data::Base64("hello".to_string());
            let write = RecordsWrite::default();

            let message = Message {
                attestation: None,
                authorization: None,
                data: Some(data),
                descriptor: Descriptor::RecordsWrite(write),
                record_id: None,
            };

            let did = "did:example:123";

            // Test put and get
            let cid = store
                .put(did, &message)
                .await
                .expect("Failed to put message");

            let got = store
                .get(did, &cid.to_string())
                .await
                .expect("Failed to get message");

            assert_eq!(message, got);

            // Test delete
            store
                .delete(did, cid.to_string())
                .await
                .expect("Failed to delete message");

            let got = store.get(did, &cid.to_string()).await;

            assert!(got.is_err());
        }

        pub async fn test_get_missing(store: impl MessageStore) {
            let did = "did:example:123";

            let got = store.get(did, "missing").await;

            assert!(got.is_err());
        }

        pub async fn test_delete_missing(store: impl MessageStore) {
            let did = "did:example:123";

            let got = store.delete(did, "missing".to_string()).await;
            assert!(got.is_err());
        }

        pub async fn test_delete_wrong_tenant(store: impl MessageStore) {
            let did = "did:example:123";

            let data = Data::Base64("hello".to_string());
            let write = RecordsWrite::default();

            let message = Message {
                attestation: None,
                authorization: None,
                data: Some(data),
                descriptor: Descriptor::RecordsWrite(write),
                record_id: None,
            };

            let cid = store
                .put(did, &message)
                .await
                .expect("Failed to put message");

            // Delete returns OK, but message should not be deleted
            let res = store.delete("wrong", cid.to_string()).await;
            assert!(res.is_ok());

            let got = store
                .get(did, &cid.to_string())
                .await
                .expect("Failed to get message");

            assert_eq!(message, got);
        }
    }
}
