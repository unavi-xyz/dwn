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
    #[error("Failed to interact with data store: {0}")]
    DataStoreError(#[from] DataStoreError),
}

pub trait MessageStore {
    fn delete(
        &self,
        tenant: &str,
        cid: String,
        data_store: &impl DataStore,
    ) -> impl Future<Output = Result<(), MessageStoreError>>;
    fn get(
        &self,
        tenant: &str,
        cid: &str,
        data_store: &impl DataStore,
    ) -> impl Future<Output = Result<Message, MessageStoreError>>;
    fn put(
        &self,
        tenant: &str,
        message: Message,
        data_store: &impl DataStore,
    ) -> impl Future<Output = Result<Cid, MessageStoreError>>;
    fn query(
        &self,
        tenant: &str,
        filter: Filter,
    ) -> impl Future<Output = Result<Vec<Message>, MessageStoreError>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_data_store {
        ($($name:ident: $type:ty,)*) => {
        $(
            mod $name {
                use super::*;

                #[tokio::test]
                async fn test_all_methods() {
                    let store = <$type>::new().await.expect("Failed to create store");
                    super::data::test_all_methods(store).await;
                }
            }
        )*
        }
    }

    macro_rules! test_message_store {
        ($($name:ident: $type:ty,)*) => {
        $(
            mod $name {
                use super::*;

                #[tokio::test]
                async fn test_all_methods() {
                    let store = <$type>::new().await.expect("Failed to create store");
                    super::message::test_all_methods(store).await;
                }

                #[tokio::test]
                async fn test_get_missing() {
                    let store = <$type>::new().await.expect("Failed to create store");
                    super::message::test_get_missing(store).await;
                }

                #[tokio::test]
                async fn test_delete_missing() {
                    let store = <$type>::new().await.expect("Failed to create store");
                    super::message::test_delete_missing(store).await;
                }

                #[tokio::test]
                async fn test_delete_wrong_tenant() {
                    let store = <$type>::new().await.expect("Failed to create store");
                    super::message::test_delete_wrong_tenant(store).await;
                }

                #[tokio::test]
                async fn test_strip_data() {
                    let store = <$type>::new().await.expect("Failed to create store");
                    super::message::test_strip_data(store).await;
                }

                #[tokio::test]
                async fn test_query() {
                    let store = <$type>::new().await.expect("Failed to create store");
                    super::message::test_query(store).await;
                }
            }
        )*
        }
    }

    #[cfg(feature = "surrealdb")]
    test_data_store! {
        surrealdb_data: SurrealDB,
    }

    #[cfg(feature = "surrealdb")]
    test_message_store! {
        surrealdb_message: SurrealDB,
    }

    pub mod data {
        use super::*;

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
            let got = store.get(&cid).await.expect("Failed to get data");

            assert!(got.is_none());
        }
    }

    pub mod message {
        use super::*;
        use crate::{
            message::{
                builder::MessageBuilder,
                descriptor::{FilterDateSort, RecordsWrite},
            },
            util::DidKey,
        };

        pub async fn test_all_methods(store: impl MessageStore) {
            let data_store = SurrealDB::new().await.expect("Failed to create data store");
            let did_key = DidKey::new().expect("Failed to generate DID key");

            let message = MessageBuilder::new::<RecordsWrite>()
                .authorize(did_key.kid.clone(), &did_key.jwk)
                .build()
                .expect("Failed to build message");

            // Test put and get
            let cid = store
                .put(&did_key.did, message.clone(), &data_store)
                .await
                .expect("Failed to put message");

            let got = store
                .get(&did_key.did, &cid.to_string(), &data_store)
                .await
                .expect("Failed to get message");

            assert_eq!(message, got);

            // Test query
            let filter = Filter {
                attester: Some(did_key.did.clone()),
                ..Default::default()
            };

            let got = store
                .query(&did_key.did, filter)
                .await
                .expect("Failed to query messages");

            assert_eq!(1, got.len());
            assert_eq!(got[0], message);

            // Test delete
            store
                .delete(&did_key.did, cid.to_string(), &data_store)
                .await
                .expect("Failed to delete message");

            let got = store.get(&did_key.did, &cid.to_string(), &data_store).await;

            assert!(got.is_err());
        }

        pub async fn test_get_missing(store: impl MessageStore) {
            let data_store = SurrealDB::new().await.expect("Failed to create data store");
            let did_key = DidKey::new().expect("Failed to generate DID key");

            let got = store.get(&did_key.did, "missing", &data_store).await;

            assert!(got.is_err());
        }

        pub async fn test_delete_missing(store: impl MessageStore) {
            let data_store = SurrealDB::new().await.expect("Failed to create data store");
            let did_key = DidKey::new().expect("Failed to generate DID key");

            let got = store
                .delete(&did_key.did, "missing".to_string(), &data_store)
                .await;
            assert!(got.is_ok());
        }

        pub async fn test_delete_wrong_tenant(store: impl MessageStore) {
            let data_store = SurrealDB::new().await.expect("Failed to create data store");
            let did_key = DidKey::new().expect("Failed to generate DID key");

            let message = MessageBuilder::new::<RecordsWrite>()
                .authorize(did_key.kid.clone(), &did_key.jwk)
                .build()
                .expect("Failed to build message");

            let cid = store
                .put(&did_key.did, message.clone(), &data_store)
                .await
                .expect("Failed to put message");

            // Delete returns OK, but message should not be deleted
            store
                .delete("wrong", cid.to_string(), &data_store)
                .await
                .expect("Failed to delete message");

            let got = store
                .get(&did_key.did, &cid.to_string(), &data_store)
                .await
                .expect("Failed to get message");

            assert_eq!(message, got);
        }

        pub async fn test_strip_data(store: impl MessageStore) {
            let data_store = SurrealDB::new().await.expect("Failed to create data store");
            let did_key = DidKey::new().expect("Failed to generate DID key");

            let mut message = MessageBuilder::new::<RecordsWrite>()
                .authorize(did_key.kid.clone(), &did_key.jwk)
                .build()
                .expect("Failed to build message");

            let cid = store
                .put(&did_key.did, message.clone(), &data_store)
                .await
                .expect("Failed to put message");

            message.data = None;

            let got = store
                .get(&did_key.did, &cid.to_string(), &data_store)
                .await
                .expect("Failed to get message");

            assert_eq!(message, got);
        }

        pub async fn test_query(store: impl MessageStore) {
            let data_store = SurrealDB::new().await.expect("Failed to create data store");
            let did_key = DidKey::new().expect("Failed to generate DID key");

            let message1 = MessageBuilder::new::<RecordsWrite>()
                .authorize(did_key.kid.clone(), &did_key.jwk)
                .build()
                .expect("Failed to build message");

            let mut message2 = MessageBuilder::new::<RecordsWrite>()
                .authorize(did_key.kid.clone(), &did_key.jwk)
                .build()
                .expect("Failed to build message");

            message2.record_id = "record2".to_string();

            store
                .put(&did_key.did, message1.clone(), &data_store)
                .await
                .expect("Failed to put message");

            store
                .put(&did_key.did, message2.clone(), &data_store)
                .await
                .expect("Failed to put message");

            // Query all messages
            {
                let filter = Filter::default();

                let got = store
                    .query(&did_key.did, filter)
                    .await
                    .expect("Failed to query messages");

                assert_eq!(2, got.len());
                assert!(got.contains(&message1));
                assert!(got.contains(&message2));
            }

            // Query specific message
            {
                let filter = Filter {
                    record_id: Some(message1.record_id.clone()),
                    ..Default::default()
                };

                let got = store
                    .query(&did_key.did, filter)
                    .await
                    .expect("Failed to query messages");

                assert_eq!(got.len(), 1);
                assert_eq!(got[0], message1);
            }

            // Query sort filter
            {
                let filter = Filter {
                    date_sort: Some(FilterDateSort::CreatedDescending),
                    ..Default::default()
                };

                let got = store
                    .query(&did_key.did, filter)
                    .await
                    .expect("Failed to query messages");

                assert_eq!(got.len(), 2);
                assert_eq!(got[0], message2);
                assert_eq!(got[1], message1);

                let filter = Filter {
                    date_sort: Some(FilterDateSort::CreatedAscending),
                    ..Default::default()
                };

                let got = store
                    .query(&did_key.did, filter)
                    .await
                    .expect("Failed to query messages");

                assert_eq!(got.len(), 2);
                assert_eq!(got[0], message1);
                assert_eq!(got[1], message2);

                let filter = Filter {
                    date_sort: Some(FilterDateSort::PublishedAscending),
                    ..Default::default()
                };

                let got = store
                    .query(&did_key.did, filter)
                    .await
                    .expect("Failed to query messages");

                assert_eq!(got.len(), 2);
                assert_eq!(got[0], message1);
                assert_eq!(got[1], message2);

                let filter = Filter {
                    date_sort: Some(FilterDateSort::PublishedDescending),
                    ..Default::default()
                };

                let got = store
                    .query(&did_key.did, filter)
                    .await
                    .expect("Failed to query messages");

                assert_eq!(got.len(), 2);
                assert_eq!(got[0], message2);
                assert_eq!(got[1], message1);
            }
        }
    }
}
