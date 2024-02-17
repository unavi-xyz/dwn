use libipld::{Block, Cid, DefaultParams};
use surrealdb::sql::{Id, Table, Thing};
use thiserror::Error;

use crate::{
    message::{DecodeError, EncodeError, Message},
    store::MessageStore,
};

use super::{
    model::{CreateMessage, GetMessage},
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
    #[error("Failed to create block {0}")]
    CreateBlockError(anyhow::Error),
    #[error("Failed to interact with SurrealDB: {0}")]
    GetDbError(anyhow::Error),
}

impl MessageStore for SurrealDB {
    type Error = MessageStoreError;

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), Self::Error> {
        let id = Thing::from((Table::from(tenant).to_string(), Id::String(cid)));

        let encoded_message: Option<GetMessage> = self.db.select(id.clone()).await?;

        if let Some(msg) = encoded_message {
            if msg.tenant != tenant {
                return Err(Self::Error::NotFound);
            }

            self.db.delete::<Option<CreateMessage>>(id).await?;
        }

        Ok(())
    }

    async fn get(&self, tenant: &str, cid: &str) -> Result<Message, Self::Error> {
        let id = Thing::from((Table::from(tenant).to_string(), Id::String(cid.to_string())));

        let encoded_message: GetMessage = self
            .db
            .select(id)
            .await?
            .ok_or_else(|| Self::Error::NotFound)?;

        let cid = Cid::try_from(cid)?;
        let block = Block::<DefaultParams>::new(cid, encoded_message.message)
            .map_err(|e| Self::Error::CreateBlockError(e))?;

        let message = Message::decode_block(block)?;

        Ok(message)
    }

    async fn put(&self, tenant: &str, message: &Message) -> Result<Cid, Self::Error> {
        let db = self
            .message_db()
            .await
            .map_err(|e| Self::Error::GetDbError(e))?;

        let block = message.encode_block()?;
        let cid = block.cid();

        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        db.create::<Option<GetMessage>>(id)
            .content(CreateMessage {
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

    async fn store() -> SurrealDB {
        SurrealDB::new().await.expect("Failed to create SurrealDB")
    }

    #[tokio::test]
    async fn test_all_methods() {
        let store = store().await;
        crate::store::tests::message::test_all_methods(store).await;
    }

    #[tokio::test]
    async fn test_get_missing() {
        let store = store().await;
        crate::store::tests::message::test_get_missing(store).await;
    }

    #[tokio::test]
    async fn test_delete_missing() {
        let store = store().await;
        crate::store::tests::message::test_delete_missing(store).await;
    }

    #[tokio::test]
    async fn test_delete_wrong_tenant() {
        let store = store().await;
        crate::store::tests::message::test_delete_wrong_tenant(store).await;
    }
}
