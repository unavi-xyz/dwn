use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use libipld::Cid;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Table, Thing};
use time::OffsetDateTime;
use tracing::warn;

use crate::{
    message::{
        data::Data,
        descriptor::{Descriptor, Filter, FilterDateSort},
        Message,
    },
    store::{DataStore, MessageStore, MessageStoreError},
    util::encode_cbor,
};

use super::SurrealDB;

const DATA_REF_TABLE_NAME: &str = "data_cid_refs";
const MESSAGE_TABLE_NAME: &str = "messages";

impl MessageStore for SurrealDB {
    async fn delete(
        &self,
        tenant: &str,
        cid: String,
        data_store: &impl DataStore,
    ) -> Result<(), MessageStoreError> {
        let db = self
            .message_db()
            .await
            .map_err(MessageStoreError::BackendError)?;

        let id = Thing::from((
            Table::from(MESSAGE_TABLE_NAME.to_string()).to_string(),
            Id::String(cid),
        ));

        let message: Option<DbMessage> = db
            .select(id.clone())
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

        let message = match message {
            Some(message) => message,
            None => return Ok(()),
        };

        // Ensure the message belongs to the tenant.
        if message.tenant != tenant {
            return Err(MessageStoreError::NotFound);
        }

        // Delete the message.
        db.delete::<Option<DbMessage>>(id)
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

        if let Some(data_cid) = message.data_cid {
            let id = Thing::from((
                Table::from(DATA_REF_TABLE_NAME).to_string(),
                Id::String(data_cid.to_string()),
            ));

            let db_data_ref: Option<DbDataCidRefs> = db
                .select(id.clone())
                .await
                .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

            if let Some(db_data_ref) = db_data_ref {
                if db_data_ref.ref_count > 1 {
                    // Decrement the reference count for the data CID.
                    db.update::<Option<DbDataCidRefs>>(id)
                        .content(DbDataCidRefs {
                            ref_count: db_data_ref.ref_count - 1,
                        })
                        .await
                        .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;
                } else {
                    // Delete the data if this is the only reference.
                    db.delete::<Option<DbDataCidRefs>>(id)
                        .await
                        .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

                    data_store.delete(data_cid).await?;
                }
            }
        }

        Ok(())
    }

    async fn put(
        &self,
        tenant: String,
        mut message: Message,
        data_store: &impl DataStore,
    ) -> Result<Cid, MessageStoreError> {
        let mut data_cid = None;

        if let Some(data) = message.data.take() {
            let db = self
                .message_db()
                .await
                .map_err(MessageStoreError::BackendError)?;

            let cid = data.cid()?.to_string();

            let id = Thing::from((
                Table::from(DATA_REF_TABLE_NAME).to_string(),
                Id::String(cid.clone()),
            ));

            // Get the current data CID object.
            let db_data_ref: Option<DbDataCidRefs> = db
                .select(id.clone())
                .await
                .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

            if let Some(db_data_ref) = db_data_ref {
                // Add one to the reference count.
                db.update::<Option<DbDataCidRefs>>(id)
                    .content(DbDataCidRefs {
                        ref_count: db_data_ref.ref_count + 1,
                    })
                    .await
                    .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;
            } else {
                // Create a new data CID object.
                db.create::<Option<DbDataCidRefs>>(id)
                    .content(DbDataCidRefs { ref_count: 1 })
                    .await
                    .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

                let bytes = match data {
                    Data::Base64(data) => match URL_SAFE_NO_PAD.decode(data.as_bytes()) {
                        Ok(bytes) => bytes,
                        Err(err) => {
                            warn!("Failed to decode base64 data: {}", err);
                            data.into()
                        }
                    },
                    Data::Encrypted(data) => serde_json::to_vec(&data).map_err(|err| {
                        MessageStoreError::BackendError(anyhow!(
                            "Failed to serialize encrypted data: {}",
                            err
                        ))
                    })?,
                };

                // Store data in the data store.
                data_store.put(cid.clone(), bytes).await?;
            }

            data_cid = Some(cid);
        }

        let cbor = encode_cbor(&message)?;
        let message_cid = cbor.cid();

        let db = self
            .message_db()
            .await
            .map_err(MessageStoreError::BackendError)?;

        // Store the message.
        let id = Thing::from((
            Table::from(MESSAGE_TABLE_NAME.to_string()).to_string(),
            Id::String(message_cid.to_string()),
        ));

        let record_id = message.record_id.clone();

        let date_created = match &message.descriptor {
            Descriptor::RecordsDelete(desc) => Some(desc.message_timestamp),
            Descriptor::RecordsWrite(desc) => Some(desc.message_timestamp),
            _ => None,
        };

        let date_created = date_created.unwrap_or_else(OffsetDateTime::now_utc);

        let date_published = match &message.descriptor {
            Descriptor::RecordsWrite(desc) => desc.date_published,
            _ => None,
        };

        let date_published = date_published.unwrap_or_else(OffsetDateTime::now_utc);

        db.create::<Option<DbMessage>>(id)
            .content(DbMessage {
                data_cid,
                date_created,
                date_published,
                message,
                record_id,
                tenant,
            })
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

        Ok(*message_cid)
    }

    async fn query(&self, tenant: &str, filter: Filter) -> Result<Vec<Message>, MessageStoreError> {
        let db = self
            .message_db()
            .await
            .map_err(MessageStoreError::BackendError)?;

        let mut conditions = vec!["(tenant = $tenant OR message.descriptor.published = true)"];

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
            .bind(("table", Table::from(MESSAGE_TABLE_NAME.to_string())))
            .bind(("tenant", tenant))
            .bind(("record_id", filter.record_id))
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

        let mut db_messages: Vec<DbMessage> = res
            .take(0)
            .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

        if let Some(sort) = filter.date_sort {
            match sort {
                FilterDateSort::CreatedAscending => {
                    db_messages.sort_by(|a, b| a.date_created.cmp(&b.date_created));
                }
                FilterDateSort::CreatedDescending => {
                    db_messages.sort_by(|a, b| b.date_created.cmp(&a.date_created));
                }
                FilterDateSort::PublishedAscending => {
                    db_messages.sort_by(|a, b| a.date_published.cmp(&b.date_published));
                }
                FilterDateSort::PublishedDescending => {
                    db_messages.sort_by(|a, b| b.date_published.cmp(&a.date_published));
                }
            }
        }

        Ok(db_messages
            .into_iter()
            .map(|db_message| db_message.message)
            .collect())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DbDataCidRefs {
    ref_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct DbMessage {
    data_cid: Option<String>,
    date_created: OffsetDateTime,
    date_published: OffsetDateTime,
    message: Message,
    record_id: String,
    tenant: String,
}