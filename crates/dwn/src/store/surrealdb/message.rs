use std::collections::HashMap;

use anyhow::anyhow;
use libipld::Cid;
use semver::Version;
use serde::{Deserialize, Serialize};
use surrealdb::{
    sql::{Id, Table, Thing},
    Connection,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tracing::debug;

use crate::{
    encode::encode_cbor,
    message::{
        descriptor::{
            protocols::{ActionCan, ActionWho, ProtocolsFilter},
            records::{FilterDateCreated, FilterDateSort, RecordsFilter},
            Descriptor,
        },
        Message,
    },
    store::{DataStore, MessageStore, MessageStoreError},
};

use super::{ql::Conditions, SurrealStore};

const DATA_REF_TABLE: &str = "data_refs";
const MSG_TABLE: &str = "messages";

impl<T: Connection> MessageStore for SurrealStore<T> {
    async fn delete(
        &self,
        tenant: &str,
        cid: &str,
        data_store: &impl DataStore,
    ) -> Result<(), MessageStoreError> {
        let db = self
            .message_db(tenant)
            .await
            .map_err(MessageStoreError::Backend)?;

        let message: Option<DbMessage> = db
            .select((MSG_TABLE, cid))
            .await
            .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

        let message = match message {
            Some(message) => message,
            None => return Ok(()),
        };

        // Ensure the message belongs to the tenant.
        if message.tenant != tenant {
            return Err(MessageStoreError::NotFound);
        }

        // Delete the message.
        db.delete::<Option<DbMessage>>((MSG_TABLE, cid))
            .await
            .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

        if let Some(data_cid) = message.data_cid {
            let db_data_ref: Option<DataRefs> = db
                .select((DATA_REF_TABLE, &data_cid))
                .await
                .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

            if let Some(db_data_ref) = db_data_ref {
                if db_data_ref.ref_count > 1 {
                    // Decrement the reference count for the data CID.
                    db.update::<Option<DataRefs>>((DATA_REF_TABLE, &data_cid))
                        .content(DataRefs {
                            ref_count: db_data_ref.ref_count - 1,
                        })
                        .await
                        .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;
                } else {
                    // Delete the data if this is the only reference.
                    db.delete::<Option<DataRefs>>((DATA_REF_TABLE, &data_cid))
                        .await
                        .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

                    data_store.delete(&data_cid).await?;
                }
            }
        }

        Ok(())
    }

    async fn put(
        &self,
        authorized: bool,
        tenant: String,
        mut message: Message,
        data_store: &impl DataStore,
    ) -> Result<Cid, MessageStoreError> {
        let mut data_cid = None;

        // TODO: Only store data in data store if over a certain size.

        if let Some(data) = message.data.take() {
            let db = self.data_db().await.map_err(MessageStoreError::Backend)?;

            // Check if the data is already stored.
            let cid = data.cid()?.to_string();

            let db_data_ref: Option<DataRefs> = db
                .select((DATA_REF_TABLE, &cid))
                .await
                .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

            if let Some(db_data_ref) = db_data_ref {
                // Add one to the reference count.
                db.update::<Option<DataRefs>>((DATA_REF_TABLE, &cid))
                    .content(DataRefs {
                        ref_count: db_data_ref.ref_count + 1,
                    })
                    .await
                    .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;
            } else {
                // Create a new data CID object.
                db.create::<Option<DataRefs>>((DATA_REF_TABLE, &cid))
                    .content(DataRefs { ref_count: 1 })
                    .await
                    .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

                // Store data in the data store.
                data_store.put(cid.clone(), data.try_into()?).await?;
            }

            data_cid = Some(cid);
        }

        let db = self
            .message_db(&tenant)
            .await
            .map_err(MessageStoreError::Backend)?;

        let cbor = encode_cbor(&message)?;
        let message_cid = cbor.cid();

        let mut context = Vec::new();

        if let Some(context_id) = message.context_id.clone() {
            for parent_record_id in context_id.split('/').rev().skip(1) {
                let parent_records = self
                    .query_records(
                        tenant.clone(),
                        message.author(),
                        authorized,
                        RecordsFilter {
                            record_id: Some(parent_record_id.to_string()),
                            date_sort: Some(FilterDateSort::CreatedAscending),
                            ..Default::default()
                        },
                    )
                    .await?;

                let parent_msg = parent_records.first().ok_or(MessageStoreError::NotFound)?;

                let parent_cbor = encode_cbor(&parent_msg)?;
                let parent_cid = parent_cbor.cid();

                context.push(Thing::from((
                    Table::from(MSG_TABLE.to_string()).to_string(),
                    Id::String(parent_cid.to_string()),
                )));
            }
        }

        // Parse protocol.
        let mut can_read = None;

        if let Descriptor::RecordsWrite(descriptor) = &message.descriptor {
            if let Some(protocol) = descriptor.protocol.clone() {
                let protocols = self
                    .query_protocols(
                        tenant.clone(),
                        authorized,
                        ProtocolsFilter {
                            protocol,
                            versions: vec![descriptor
                                .protocol_version
                                .clone()
                                .unwrap_or(Version::new(0, 0, 0))],
                        },
                    )
                    .await?;

                let configure_msg = protocols.first().ok_or(MessageStoreError::NotFound)?;
                let definition = match &configure_msg.descriptor {
                    Descriptor::ProtocolsConfigure(config) => config.definition.clone(),
                    _ => unreachable!(),
                };

                if let Some(definition) = definition {
                    if let Some(path) = &descriptor.protocol_path {
                        let structure = definition
                            .get_structure(path)
                            .ok_or(MessageStoreError::NotFound)?;

                        let mut can_read_dids = Vec::new();
                        let mut set_can_read = true;

                        for action in &structure.actions {
                            if !action.can.contains(&ActionCan::Read) {
                                continue;
                            }

                            match &action.who {
                                ActionWho::Anyone => {
                                    // Ignore other actions, keep `can_read` as `None`.
                                    set_can_read = false;
                                    break;
                                }
                                ActionWho::Author => {
                                    if let Some(of) = &action.of {
                                        // `of` will always be a subset of `path`.
                                        // Get the extra parts of `path` and count them.
                                        // That is how far up `context` we have to go.
                                        let mut extra = path.strip_prefix(of).unwrap();

                                        if extra.starts_with('/') {
                                            extra = extra.strip_prefix('/').unwrap();
                                        }

                                        let count = extra.split('/').count();

                                        let id = context[count - 1].to_string();

                                        let message: Option<DbMessage> =
                                            db.select((MSG_TABLE, id)).await.map_err(|err| {
                                                MessageStoreError::Backend(anyhow!(err))
                                            })?;

                                        let message = message.ok_or(MessageStoreError::NotFound)?;

                                        if let Some(author) = message.author {
                                            can_read_dids.push(author);
                                        } else {
                                            debug!("Protocol reference has no author?");
                                        }
                                    } else {
                                        // Author can always read their own record.
                                        // Nothing needs to happen here.
                                    }
                                }
                                ActionWho::Recipient => {
                                    if let Some(of) = &action.of {
                                        // `of` will always be a subset of `path`.
                                        // Get the extra parts of `path` and count them.
                                        // That is how far up `context` we have to go.
                                        let mut extra = path.strip_prefix(of).unwrap();

                                        if extra.starts_with('/') {
                                            extra = extra.strip_prefix('/').unwrap();
                                        }

                                        let count = extra.split('/').count();

                                        // Go one level higher to get the recipient.
                                        // For root level structures this is the tenant.
                                        if count < context.len() {
                                            let id = &context[count].to_string();

                                            let message: Option<DbMessage> =
                                                db.select((MSG_TABLE, id)).await.map_err(
                                                    |err| MessageStoreError::Backend(anyhow!(err)),
                                                )?;

                                            let message =
                                                message.ok_or(MessageStoreError::NotFound)?;

                                            if let Some(author) = message.author {
                                                can_read_dids.push(author);
                                            } else {
                                                debug!("Protocol reference has no author?");
                                            }
                                        } else {
                                            can_read_dids.push(tenant.clone());
                                        };
                                    } else {
                                        can_read_dids.push(tenant.clone());
                                    }
                                }
                            }
                        }

                        if set_can_read {
                            can_read = Some(can_read_dids);
                        }
                    }
                }
            }
        }

        let record_id = message.record_id.clone();

        let message_timestamp = match &message.descriptor {
            Descriptor::RecordsDelete(desc) => Some(desc.message_timestamp),
            Descriptor::RecordsWrite(desc) => Some(desc.message_timestamp),
            _ => None,
        };

        let message_timestamp = message_timestamp.unwrap_or_else(OffsetDateTime::now_utc);

        let date_published = match &message.descriptor {
            Descriptor::RecordsWrite(desc) => desc.date_published,
            _ => None,
        };

        let date_published = date_published.unwrap_or_else(OffsetDateTime::now_utc);

        let author = message.author().map(|a| a.to_string());

        // TODO: When updating a record, if data changes delete old data / decrement ref

        db.create::<Option<DbMessage>>((MSG_TABLE, message_cid.to_string()))
            .content(DbMessage {
                author,
                can_read,
                context,
                data_cid,
                date_published,
                message,
                message_timestamp,
                record_id,
                tenant,
            })
            .await
            .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

        Ok(*message_cid)
    }

    async fn query_protocols(
        &self,
        tenant: String,
        authorized: bool,
        filter: ProtocolsFilter,
    ) -> Result<Vec<Message>, MessageStoreError> {
        let db = self
            .message_db(&tenant)
            .await
            .map_err(MessageStoreError::Backend)?;

        let mut conditions = Conditions::new_and();
        conditions.add("message.descriptor.interface = 'Protocols'".to_string());
        conditions.add("message.descriptor.method = 'Configure'".to_string());
        conditions.add("message.descriptor.definition.protocol = $protocol".to_string());

        if !filter.versions.is_empty() {
            conditions.add("message.descriptor.protocolVersion IN $versions".to_string());
        }

        if !authorized {
            conditions.add("message.descriptor.definition.published = true".to_string());
        }

        let condition_statement = if conditions.is_empty() {
            "".to_string()
        } else {
            format!(" WHERE {}", conditions)
        };

        let query = db
            .query(format!(
                "SELECT * FROM type::table($table){}",
                condition_statement
            ))
            .bind(("table", Table::from(MSG_TABLE.to_string())))
            .bind(("protocol", filter.protocol))
            .bind(("versions", filter.versions));

        let mut res = query
            .await
            .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

        let mut db_messages: Vec<DbMessage> = res
            .take(0)
            .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

        db_messages.sort_by(|a, b| b.message_timestamp.cmp(&a.message_timestamp));

        Ok(db_messages
            .into_iter()
            .map(|db_message| db_message.message)
            .collect())
    }

    async fn query_records(
        &self,
        tenant: String,
        author: Option<String>,
        authorized: bool,
        filter: RecordsFilter,
    ) -> Result<Vec<Message>, MessageStoreError> {
        let db = self
            .message_db(&tenant)
            .await
            .map_err(MessageStoreError::Backend)?;

        let mut binds = HashMap::new();
        binds.insert("tenant", tenant);

        let mut conditions = Conditions::new_and();
        conditions.add("message.descriptor.interface = 'Records'".to_string());

        {
            // TODO: Do we need to verify author?
            let mut can_read = Conditions::new_or();
            can_read.add("can_read = NONE".to_string());

            if author.is_some() {
                can_read.add("can_read CONTAINS $author".to_string());
                can_read.add("author = $author".to_string());
            }

            conditions.add(can_read.to_string());
        }

        if !authorized {
            conditions.add("message.descriptor.published = true".to_string());
        }

        if let Some(protocol_version) = filter.protocol_version {
            binds.insert("protocol_version", protocol_version.to_string());
            conditions.add("message.descriptor.protocolVersion = $protocol_version".to_string());
        }

        if let Some(record_id) = filter.record_id {
            binds.insert("record_id", record_id.to_string());
            conditions.add("record_id = $record_id".to_string());
        }

        if let Some(FilterDateCreated { from, to }) = filter.message_timestamp {
            if let Some(from) = from {
                let from = from.format(&Rfc3339).map_err(|err| {
                    MessageStoreError::Backend(anyhow!("Failed to format date: {}", err))
                })?;
                binds.insert("from", from.to_string());
                conditions.add("message_timestamp >= $from".to_string());
            }

            if let Some(to) = to {
                let to = to.format(&Rfc3339).map_err(|err| {
                    MessageStoreError::Backend(anyhow!("Failed to format date: {}", err))
                })?;
                binds.insert("to", to.to_string());
                conditions.add("message_timestamp <= $to".to_string());
            }
        };

        let condition_statement = if conditions.is_empty() {
            "".to_string()
        } else {
            format!(" WHERE {}", conditions)
        };

        let sort = match filter.date_sort {
            Some(FilterDateSort::CreatedAscending) => "message_timestamp ASC".to_string(),
            Some(FilterDateSort::CreatedDescending) => "message_timestamp DESC".to_string(),
            Some(FilterDateSort::PublishedAscending) => "date_published ASC".to_string(),
            Some(FilterDateSort::PublishedDescending) => "date_published DESC".to_string(),
            None => "".to_string(),
        };

        let sort = if sort.is_empty() {
            "".to_string()
        } else {
            format!(" ORDER BY {}", sort)
        };

        let query_string = format!(
            "SELECT * FROM type::table($table){}{}",
            condition_statement, sort
        );
        debug!("{}", query_string);

        let mut query = db
            .query(query_string)
            .bind(("table", MSG_TABLE))
            .bind(("author", author));

        for (key, value) in binds {
            query = query.bind((key, value));
        }

        let mut res = query
            .await
            .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

        let db_messages: Vec<DbMessage> = res
            .take(0)
            .map_err(|err| MessageStoreError::Backend(anyhow!(err)))?;

        Ok(db_messages
            .into_iter()
            .map(|db_message| db_message.message)
            .collect())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DataRefs {
    ref_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct DbMessage {
    // Author DID of the message.
    author: Option<String>,
    // Restrict reading to this list of DIDs, created from the message protocol.
    // If `None`, normal read rules apply.
    can_read: Option<Vec<String>>,
    // Protocol context.
    context: Vec<Thing>,
    data_cid: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    date_published: OffsetDateTime,
    message: Message,
    #[serde(with = "time::serde::rfc3339")]
    message_timestamp: OffsetDateTime,
    record_id: String,
    tenant: String,
}
