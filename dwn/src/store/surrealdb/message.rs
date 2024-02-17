use libipld::{Block, Cid, DefaultParams};
use surrealdb::sql::{Id, Table, Thing};
use thiserror::Error;

use crate::{
    message::{DecodeError, EncodeError, Message},
    store::MessageStore,
};

use super::{
    model::{CreateEncodedMessage, GetEncodedMessage},
    SurrealDB,
};

#[derive(Error, Debug)]
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

impl MessageStore for SurrealDB {
    type Error = MessageStoreError;

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), Self::Error> {
        let id = Thing::from((Table::from(tenant).to_string(), Id::String(cid)));

        let encoded_message: Option<GetEncodedMessage> = self.db.select(id.clone()).await?;

        if let Some(msg) = encoded_message {
            if msg.tenant != tenant {
                return Err(Self::Error::NotFound);
            }

            self.db.delete::<Option<CreateEncodedMessage>>(id).await?;
        }

        Ok(())
    }

    async fn get(&self, tenant: &str, cid: &str) -> Result<Message, Self::Error> {
        let id = Thing::from((Table::from(tenant).to_string(), Id::String(cid.to_string())));

        let encoded_message: GetEncodedMessage = self
            .db
            .select(id)
            .await?
            .ok_or_else(|| Self::Error::NotFound)?;

        let cid = Cid::try_from(cid)?;
        let block = Block::<DefaultParams>::new(cid, encoded_message.message)?;

        let message = Message::decode_block(block)?;

        Ok(message)
    }

    async fn put(&self, tenant: &str, message: &Message) -> Result<Cid, Self::Error> {
        let db = self.message_db().await?;

        let block = message.encode_block()?;
        let cid = block.cid();

        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        db.create::<Option<GetEncodedMessage>>(id)
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
    async fn test_all_methods() {
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

        // Test put and get
        let cid = surreal
            .put(did, &message)
            .await
            .expect("Failed to put message");

        let got = surreal
            .get(did, &cid.to_string())
            .await
            .expect("Failed to get message");

        assert_eq!(message, got);

        // Test delete
        surreal
            .delete(did, cid.to_string())
            .await
            .expect("Failed to delete message");

        let got = surreal.get(did, &cid.to_string()).await;

        assert!(got.is_err());
    }

    #[tokio::test]
    async fn test_get_missing() {
        let surreal = SurrealDB::new().await.expect("Failed to create SurrealDB");

        let did = "did:example:123";

        let got = surreal.get(did, "missing").await;

        assert!(got.is_err());
    }

    #[tokio::test]
    async fn test_delete_missing() {
        let surreal = SurrealDB::new().await.expect("Failed to create SurrealDB");

        let did = "did:example:123";

        let got = surreal.delete(did, "missing".to_string()).await;
        assert!(got.is_err());
    }

    #[tokio::test]
    async fn test_delete_wrong_tenant() {
        let surreal = SurrealDB::new().await.expect("Failed to create SurrealDB");

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

        let cid = surreal
            .put(did, &message)
            .await
            .expect("Failed to put message");

        // Delete returns OK, but message should not be deleted
        let res = surreal.delete("wrong", cid.to_string()).await;
        assert!(res.is_ok());

        let got = surreal
            .get(did, &cid.to_string())
            .await
            .expect("Failed to get message");

        assert_eq!(message, got);
    }
}
