use libipld::Cid;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Table, Thing};

use crate::{
    message::{descriptor::Filter, Message},
    store::{MessageStore, MessageStoreError},
};

use super::SurrealDB;

impl MessageStore for SurrealDB {
    async fn delete(&self, tenant: &str, cid: String) -> Result<(), MessageStoreError> {
        let id = Thing::from((Table::from(tenant).to_string(), Id::String(cid)));

        let message: Option<DbMessage> = self
            .db
            .select(id.clone())
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?;

        if let Some(msg) = message {
            if msg.tenant != tenant {
                return Err(MessageStoreError::NotFound);
            }

            self.db
                .delete::<Option<DbMessage>>(id)
                .await
                .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?;
        }

        Ok(())
    }

    async fn get(&self, tenant: &str, cid: &str) -> Result<Message, MessageStoreError> {
        let id = Thing::from((Table::from(tenant).to_string(), Id::String(cid.to_string())));

        let db_message: DbMessage = self
            .db
            .select(id)
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?
            .ok_or_else(|| MessageStoreError::NotFound)?;

        Ok(db_message.message)
    }

    async fn put(&self, tenant: &str, message: Message) -> Result<Cid, MessageStoreError> {
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

        db.create::<Option<DbMessage>>(id)
            .content(DbMessage {
                cid: cid.to_string(),
                message,
                tenant: tenant.to_string(),
            })
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?;

        Ok(*cid)
    }

    async fn query(
        &self,
        tenant: &str,
        _filter: Filter,
    ) -> Result<Vec<Message>, MessageStoreError> {
        let db = self
            .message_db()
            .await
            .map_err(MessageStoreError::BackendError)?;

        let query = "SELECT * FROM type::table($table);";

        let mut res = db
            .query(query)
            .bind(("table", Table::from(tenant).to_string()))
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?;

        let db_messages: Vec<DbMessage> = res
            .take(0)
            .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?;

        Ok(db_messages
            .into_iter()
            .map(|db_message| db_message.message)
            .collect())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DbMessage {
    cid: String,
    message: Message,
    tenant: String,
}
