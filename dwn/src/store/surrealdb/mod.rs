//! Message store implementation using an embedded SurrealDB database.
//! Saves to the file system when run natively, or to IndexedDB when run in the browser.

use std::sync::Arc;

use surrealdb::{
    engine::local::{Db, Mem},
    sql::{Id, Table, Thing},
    Surreal,
};

use crate::message::{CidError, Message};

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
    CidError(#[from] CidError),
    #[error("Failed to interact with SurrealDB: {0}")]
    SurrealDB(#[from] surrealdb::Error),
    #[error("Failed to encode data: {0}")]
    DataEncodeError(#[from] libipld_core::error::SerdeError),
}

pub struct SurrealDB {
    pub db: Arc<Surreal<Db>>,
}

impl SurrealDB {
    async fn new() -> Result<Self, surrealdb::Error> {
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

    fn delete(&self, tenant: &str, cid: String) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn get(&self, tenant: &str, cid: String) -> Result<Message, Self::Error> {
        unimplemented!()
    }

    async fn put(&self, tenant: &str, message: Message) -> Result<libipld::Cid, Self::Error> {
        let db = self.message_db().await?;

        let data = message.data.as_ref().ok_or(Self::Error::MissingData)?;
        let encoded_data = data.encode()?;

        let block = message.cbor_block()?;
        let cid = block.cid();

        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        db.create::<Option<GetEncodedMessage>>(id.clone())
            .content(CreateEncodedMessage {
                cid: cid.to_string(),
                encoded_data: Some(encoded_data),
                encoded_message: block.data().to_vec(),
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
    async fn test_put() {
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

        let _ = surreal
            .put(did, message)
            .await
            .expect("Failed to put message");
    }
}
