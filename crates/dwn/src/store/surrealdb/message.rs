use std::collections::HashMap;

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use libipld::Cid;
use serde::{Deserialize, Serialize};
use surrealdb::{
    sql::{Id, Table, Thing},
    Connection,
};
use time::OffsetDateTime;
use tracing::{debug, warn};

use crate::{
    encode::encode_cbor,
    message::{
        descriptor::{
            protocols::{ActionCan, ActionWho, ProtocolsFilter},
            records::{FilterDateSort, RecordsFilter},
            Descriptor,
        },
        Data, EncryptedData, Message,
    },
    store::{DataStore, MessageStore, MessageStoreError},
};

use super::{ql::Conditions, SurrealStore};

const DATA_REF_TABLE: &str = "data_cid_refs";
const MESSAGE_TABLE: &str = "messages";

impl<T: Connection> MessageStore for SurrealStore<T> {
    async fn delete(
        &self,
        tenant: &str,
        cid: String,
        data_store: &impl DataStore,
    ) -> Result<(), MessageStoreError> {
        let db = self
            .message_db(tenant)
            .await
            .map_err(MessageStoreError::BackendError)?;

        let id = Thing::from((
            Table::from(MESSAGE_TABLE.to_string()).to_string(),
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
        // TODO: How to delete messages from your dwn that are not yours?
        if message.tenant != tenant {
            return Err(MessageStoreError::NotFound);
        }

        // Delete the message.
        db.delete::<Option<DbMessage>>(id)
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

        if let Some(data_cid) = message.data_cid {
            let id = Thing::from((
                Table::from(DATA_REF_TABLE).to_string(),
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

        // TODO: Only store data in data store if over a certain size.

        if let Some(data) = message.data.take() {
            // Keep the data type, but clear the data.
            match &data {
                Data::Base64(_) => {
                    message.data = Some(Data::Base64(String::new()));
                }
                Data::Encrypted(data) => {
                    message.data = Some(Data::Encrypted(EncryptedData {
                        ciphertext: String::new(),
                        iv: data.iv.clone(),
                        tag: data.tag.clone(),
                        protected: data.protected.clone(),
                        recipients: data.recipients.clone(),
                    }));
                }
            }

            // Check if the data is already stored.
            let db = self
                .message_db(&tenant)
                .await
                .map_err(MessageStoreError::BackendError)?;

            let cid = data.cid()?.to_string();

            let id = Thing::from((
                Table::from(DATA_REF_TABLE).to_string(),
                Id::String(cid.clone()),
            ));

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
                    Data::Base64(data) => URL_SAFE_NO_PAD
                        .decode(data.as_bytes())
                        .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?,
                    Data::Encrypted(data) => URL_SAFE_NO_PAD
                        .decode(data.ciphertext.as_bytes())
                        .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?,
                };

                // Store data in the data store.
                data_store.put(cid.clone(), bytes).await?;
            }

            data_cid = Some(cid);
        }

        let cbor = encode_cbor(&message)?;
        let message_cid = cbor.cid();

        let db = self
            .message_db(&tenant)
            .await
            .map_err(MessageStoreError::BackendError)?;

        // Store the message.
        let id = Thing::from((
            Table::from(MESSAGE_TABLE.to_string()).to_string(),
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

        let author = message.author().map(|a| a.to_string());

        db.create::<Option<DbMessage>>(id)
            .content(DbMessage {
                author,
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

    async fn query_protocols(
        &self,
        tenant: String,
        authorized: bool,
        filter: ProtocolsFilter,
    ) -> Result<Vec<Message>, MessageStoreError> {
        let db = self
            .message_db(&tenant)
            .await
            .map_err(MessageStoreError::BackendError)?;

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
            .bind(("table", Table::from(MESSAGE_TABLE.to_string())))
            .bind(("protocol", filter.protocol))
            .bind(("versions", filter.versions));

        let mut res = query
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

        let mut db_messages: Vec<DbMessage> = res
            .take(0)
            .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

        db_messages.sort_by(|a, b| a.date_created.cmp(&b.date_created));

        Ok(db_messages
            .into_iter()
            .map(|db_message| db_message.message)
            .collect())
    }

    async fn query_records(
        &self,
        tenant: String,
        author: Option<&str>,
        authorized: bool,
        filter: RecordsFilter,
    ) -> Result<Vec<Message>, MessageStoreError> {
        let db = self
            .message_db(&tenant)
            .await
            .map_err(MessageStoreError::BackendError)?;

        let mut binds = HashMap::new();
        let mut conditions = Conditions::new_and();

        if !authorized {
            conditions.add("message.descriptor.published = true".to_string());
        }

        if let Some(protocol) = &filter.protocol {
            binds.insert("protocol".to_string(), protocol.to_string());
            conditions.add("message.descriptor.protocol = $protocol".to_string());

            // Get protocol.
            let mut versions = Vec::new();

            if let Some(protocol_version) = &filter.protocol_version {
                versions.push(protocol_version.clone());
            }

            let protocols = self
                .query_protocols(
                    tenant.clone(),
                    authorized,
                    ProtocolsFilter {
                        protocol: protocol.clone(),
                        versions,
                    },
                )
                .await?;

            if let Some(protocol) = protocols.first() {
                let descriptor = match &protocol.descriptor {
                    Descriptor::ProtocolsConfigure(desc) => desc,
                    _ => return Err(MessageStoreError::BackendError(anyhow!("Invalid protocol"))),
                };

                if let Some(definition) = &descriptor.definition {
                    // Enforce protocol read rules.
                    let mut who = vec![ActionWho::Anyone];

                    if authorized {
                        who.push(ActionWho::Recipient);
                    }

                    let mut protocol_conditions = Conditions::new_or();

                    for (i, (key, structure)) in definition.structure.iter().enumerate() {
                        let mut read_conditions = Conditions::new_and();

                        for action in &structure.actions {
                            if action.can != ActionCan::Read {
                                continue;
                            }

                            if let Some(of) = &action.of {
                                // TODO: Support 'of' parent structure key
                                // Will need to traverse up contextId?
                                if of != key {
                                    warn!("Read action currently unsupported: {:?}", action);
                                    continue;
                                }
                            }

                            if action.who == ActionWho::Author {
                                let mut author_conditions = Conditions::new_or();

                                if let Some(author) = &author {
                                    let bind = format!("protocol_structure_author_{}", i);
                                    binds.insert(bind.clone(), author.to_string());

                                    author_conditions.add(format!("author = ${}", bind));
                                }

                                if authorized {
                                    binds.insert("tenant".to_string(), tenant.clone());
                                    author_conditions.add("author = $tenant".to_string());
                                }

                                read_conditions.add(author_conditions.to_string());
                            } else if who.contains(&action.who) {
                                read_conditions.add("true".to_string());
                            }
                        }

                        let mut structure_conditions = Conditions::new_and();

                        let bind = format!("protocol_structure_{}", i);
                        binds.insert(bind.clone(), key.clone());
                        let eq = if read_conditions.is_empty() {
                            "!=".to_string()
                        } else {
                            "=".to_string()
                        };
                        structure_conditions
                            .add(format!("message.descriptor.protocolPath {} ${}", eq, bind));
                        structure_conditions.add(read_conditions.to_string());

                        protocol_conditions.add(structure_conditions.to_string());
                    }

                    conditions.add(protocol_conditions.to_string());
                }
            }
        }

        if let Some(protocol_version) = filter.protocol_version {
            binds.insert("protocol_version".to_string(), protocol_version.to_string());
            conditions.add("message.descriptor.protocolVersion = $protocol_version".to_string());
        }

        if let Some(record_id) = filter.record_id {
            binds.insert("record_id".to_string(), record_id.to_string());
            conditions.add("record_id = $record_id".to_string());
        }

        let condition_statement = if conditions.is_empty() {
            "".to_string()
        } else {
            format!(" WHERE {}", conditions)
        };

        let query_string = format!("SELECT * FROM type::table($table){}", condition_statement);
        debug!("Querying records: {}", query_string);

        let mut query = db
            .query(query_string)
            .bind(("table", Table::from(MESSAGE_TABLE.to_string())));

        for (key, value) in binds {
            query = query.bind((key, value));
        }

        let mut res = query
            .await
            .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

        let mut db_messages: Vec<DbMessage> = res
            .take(0)
            .map_err(|err| MessageStoreError::BackendError(anyhow!(err)))?;

        // TODO: Sort in query

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
    author: Option<String>,
    data_cid: Option<String>,
    date_created: OffsetDateTime,
    date_published: OffsetDateTime,
    message: Message,
    record_id: String,
    tenant: String,
}
