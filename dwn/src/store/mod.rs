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

    fn delete(&self, cid: &Cid) -> impl Future<Output = Result<(), Self::Error>>;
    fn get(&self, cid: &Cid) -> impl Future<Output = Result<Option<GetDataResults>, Self::Error>>;
    fn put(
        &self,
        cid: &Cid,
        value: Vec<u8>,
    ) -> impl Future<Output = Result<PutDataResults, Self::Error>>;
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

pub trait MessageStore {
    type Error: Error + Send + Sync + 'static;

    fn delete(&self, tenant: &str, cid: String) -> impl Future<Output = Result<(), Self::Error>>;
    fn get(&self, tenant: &str, cid: &str) -> impl Future<Output = Result<Message, Self::Error>>;
    fn put(
        &self,
        tenant: &str,
        message: &Message,
    ) -> impl Future<Output = Result<Cid, Self::Error>>;
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
            descriptor::{records::RecordsWrite, Descriptor},
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
