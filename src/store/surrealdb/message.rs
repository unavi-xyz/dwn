use libipld::Cid;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Table, Thing};
use time::OffsetDateTime;

use crate::{
    message::{
        descriptor::{Descriptor, Filter},
        Message,
    },
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

        if message.is_some() {
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

    async fn put(&self, tenant: &str, mut message: Message) -> Result<Cid, MessageStoreError> {
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

        message.data = None;

        let record_id = message.record_id.clone();

        let date_created = match &message.descriptor {
            Descriptor::RecordsWrite(desc) => Some(desc.date_created.clone()),
            _ => None,
        };

        let date_published = match &message.descriptor {
            Descriptor::RecordsWrite(desc) => desc.date_published.clone(),
            _ => None,
        };

        db.create::<Option<DbMessage>>(id)
            .content(DbMessage {
                date_created,
                date_published,
                message,
                record_id,
            })
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow::anyhow!(err)))?;

        Ok(*cid)
    }

    async fn query(&self, tenant: &str, filter: Filter) -> Result<Vec<Message>, MessageStoreError> {
        let db = self
            .message_db()
            .await
            .map_err(MessageStoreError::BackendError)?;

        let mut conditions = Vec::new();

        if filter.record_id.is_some() {
            conditions.push("record_id = $record_id");
        }

        let condition_statement = if conditions.is_empty() {
            "".to_string()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let query = format!("SELECT * FROM type::table($table){}", condition_statement);

        let mut res = db
            .query(query)
            .bind(("table", Table::from(tenant).to_string()))
            .bind(("record_id", filter.record_id))
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
    date_created: Option<OffsetDateTime>,
    date_published: Option<OffsetDateTime>,
    message: Message,
    record_id: String,
}
