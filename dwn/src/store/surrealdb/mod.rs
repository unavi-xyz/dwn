//! Message store implementation using an embedded SurrealDB database.
//! Saves to the file system when run natively, or to IndexedDB when run in the browser.

use std::sync::Arc;

use libipld::{Block, Cid, DefaultParams};
use surrealdb::{
    engine::local::{Db, Mem},
    sql::{Id, Table, Thing},
    Surreal,
};
use tracing::info;

use crate::message::{DecodeError, EncodeError, Message};

use self::model::{CreateEncodedMessage, GetEncodedMessage};

use super::MessageStore;

pub mod model;

const NAMESPACE: &str = "dwn";
const DBNAME: &str = "message";

#[derive(thiserror::Error, Debug)]
pub enum MessageStoreError {
    #[error("Message missing data")]
    MissingData,
    #[error("Failed to generate CID: {0}")]
    MessageEncode(#[from] EncodeError),
    #[error("Failed to interact with SurrealDB: {0}")]
    SurrealDB(#[from] surrealdb::Error),
    #[error("Failed to encode data: {0}")]
    DataEncodeError(#[from] libipld_core::error::SerdeError),
    #[error("Not found")]
    NotFound,
    #[error("Failed to decode message: {0}")]
    MessageDecodeError(#[from] DecodeError),
    #[error("Failed to generate CID: {0}")]
    Cid(#[from] libipld::cid::Error),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

pub struct SurrealDB {
    db: Arc<Surreal<Db>>,
}

impl SurrealDB {
    pub async fn new() -> Result<Self, surrealdb::Error> {
        let db = Surreal::new::<Mem>(()).await?;
        Ok(SurrealDB { db: Arc::new(db) })
    }

    pub async fn message_db(&self) -> Result<Arc<Surreal<Db>>, MessageStoreError> {
        self.db.use_ns(NAMESPACE).use_db(DBNAME).await?;
        Ok(self.db.clone())
    }
}

impl MessageStore for SurrealDB {
    type Error = MessageStoreError;

    fn delete(&self, tenant: &str, cid: &str) -> Result<(), Self::Error> {
        unimplemented!()
    }

    async fn get(&self, tenant: &str, cid: &str) -> Result<Message, Self::Error> {
        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        let encoded_message: GetEncodedMessage = self
            .db
            .select(id.clone())
            .await?
            .ok_or_else(|| Self::Error::NotFound)?;

        let cid = Cid::try_from(cid)?;
        let block = Block::<DefaultParams>::new(cid, encoded_message.message)?;

        let message = Message::decode_block(block)?;

        Ok(message)
    }

    async fn put(&self, tenant: &str, message: Message) -> Result<libipld::Cid, Self::Error> {
        let db = self.message_db().await?;

        let block = message.encode_block()?;
        let cid = block.cid();

        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        db.create::<Option<GetEncodedMessage>>(id.clone())
            .content(CreateEncodedMessage {
                cid: cid.to_string(),
                message: block.data().to_vec(),
                tenant: tenant.to_string(),
            })
            .await?;

        Ok(*cid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{
        descriptor::{records::RecordsWrite, Descriptor},
        Data, Message,
    };

    #[tokio::test]
    async fn test_put_get() {
        let surreal = SurrealDB::new().await.expect("Failed to create SurrealDB");

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

        let cid = surreal
            .put(did, message.clone())
            .await
            .expect("Failed to put message");

        let got = surreal
            .get(did, &cid.to_string())
            .await
            .expect("Failed to get message");

        assert_eq!(message, got);
    }

    #[tokio::test]
    async fn test_get_missing() {
        let surreal = SurrealDB::new().await.expect("Failed to create SurrealDB");

        let did = "did:example:123";

        let got = surreal.get(did, "missing").await;

        assert!(got.is_err());
    }
}
