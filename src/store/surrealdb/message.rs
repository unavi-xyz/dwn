use libipld::{Block, Cid, DefaultParams};
use surrealdb::sql::{Id, Table, Thing};

use crate::{
    message::Message,
    store::{MessageStore, MessageStoreError},
};

use super::{
    model::{CreateMessage, GetMessage},
    SurrealDB,
};

impl MessageStore for SurrealDB {
    async fn delete(&self, tenant: &str, cid: String) -> Result<(), MessageStoreError> {
        let id = Thing::from((Table::from(tenant).to_string(), Id::String(cid)));

        let encoded_message: Option<GetMessage> = self
            .db
            .select(id.clone())
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?;

        if let Some(msg) = encoded_message {
            if msg.tenant != tenant {
                return Err(MessageStoreError::NotFound);
            }

            self.db
                .delete::<Option<CreateMessage>>(id)
                .await
                .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?;
        }

        Ok(())
    }

    async fn get(&self, tenant: &str, cid: &str) -> Result<Message, MessageStoreError> {
        let id = Thing::from((Table::from(tenant).to_string(), Id::String(cid.to_string())));

        let encoded_message: GetMessage = self
            .db
            .select(id)
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?
            .ok_or_else(|| MessageStoreError::NotFound)?;

        let cid = Cid::try_from(cid)?;
        let block = Block::<DefaultParams>::new(cid, encoded_message.message)
            .map_err(MessageStoreError::CreateBlockError)?;

        let message = Message::decode_block(block)?;

        Ok(message)
    }

    async fn put(&self, tenant: &str, message: &Message) -> Result<Cid, MessageStoreError> {
        let db = self
            .message_db()
            .await
            .map_err(MessageStoreError::BackendError)?;

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
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?;

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