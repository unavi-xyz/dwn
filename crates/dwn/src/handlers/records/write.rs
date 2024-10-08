use std::sync::Arc;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonschema::Validator;
use reqwest::Client;
use serde_json::Value;
use tracing::debug;

use crate::{
    encode::encode_cbor,
    message::{
        descriptor::{
            protocols::{ActionCan, ActionWho, ProtocolsFilter},
            records::{FilterDateSort, RecordsFilter},
            Descriptor,
        },
        Data, DwnRequest,
    },
    reply::{MessageReply, Status, StatusReply},
    store::{DataStore, MessageStore},
    HandleMessageError,
};

#[derive(Debug, Default)]
pub struct HandleWriteOptions {
    /// Ignore the parent id, allowing the message to be written even if the parent message is not found.
    pub ignore_parent_id: bool,
}

pub async fn handle_records_write(
    client: &Client,
    data_store: &Arc<dyn DataStore>,
    message_store: &Arc<dyn MessageStore>,
    DwnRequest { target, message }: DwnRequest,
    options: HandleWriteOptions,
) -> Result<MessageReply, HandleMessageError> {
    let authorized = message.is_authorized(&target).await;

    let descriptor = match &message.descriptor {
        Descriptor::RecordsWrite(descriptor) => descriptor,
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a RecordsWrite message".to_string(),
            ))
        }
    };

    // Validate data format is present.
    if message.data.is_some() && descriptor.data_format.is_none() {
        return Err(HandleMessageError::InvalidDescriptor(
            "Data format missing".to_string(),
        ));
    }

    // Validate data CID if data present.
    if let Some(data) = &message.data {
        let cid = data.cid()?;

        let data_cid =
            descriptor
                .data_cid
                .as_deref()
                .ok_or(HandleMessageError::InvalidDescriptor(
                    "Data CID missing".to_string(),
                ))?;

        if cid.to_string() != data_cid {
            return Err(HandleMessageError::InvalidDescriptor(
                "Data CID does not match data".to_string(),
            ));
        }
    }

    // Get messages for the record.
    let messages = message_store
        .query_records(
            target.clone(),
            message.author(),
            authorized,
            RecordsFilter {
                record_id: Some(message.record_id.clone()),
                date_sort: Some(FilterDateSort::CreatedDescending),
                ..Default::default()
            },
        )
        .await?;

    let initial_entry = messages.last();

    let mut checkpoint_entry = messages
        .iter()
        .find(|m| matches!(m.descriptor, Descriptor::RecordsDelete(_)));

    if let Some(initial_entry) = initial_entry {
        if checkpoint_entry.is_none() {
            checkpoint_entry = Some(initial_entry);
        }

        // Validate schema.
        let initial_schema = match &initial_entry.descriptor {
            Descriptor::RecordsWrite(desc) => desc.schema.clone(),
            _ => {
                return Err(HandleMessageError::InvalidDescriptor(
                    "Initial entry is not a RecordsWrite message".to_string(),
                ))
            }
        };

        if initial_schema != descriptor.schema {
            return Err(HandleMessageError::InvalidDescriptor(
                "Schema does not match initial entry".to_string(),
            ));
        }
    }

    // Validate data matches schema.
    if let Some(schema_url) = &descriptor.schema {
        if let Some(Data::Base64(data)) = &message.data {
            debug!("Fetching schema: {}", schema_url);
            let schema = client.get(schema_url).send().await?.json::<Value>().await?;

            let compiled = Validator::new(&schema).map_err(|_| {
                HandleMessageError::SchemaValidation("Failed to compile schema".to_string())
            })?;

            let data = URL_SAFE_NO_PAD.decode(data)?;
            let data = serde_json::from_slice(&data)?;

            compiled.validate(&data).map_err(|_| {
                HandleMessageError::SchemaValidation("Data does not match schema".to_string())
            })?;
        } else {
            return Err(HandleMessageError::InvalidDescriptor(
                "Can not validate schema against encrypted data".to_string(),
            ));
        }
    }

    // Validate protocol rules.
    let mut can_write = authorized;

    if message.context_id.is_some()
        || descriptor.protocol.is_some()
        || descriptor.protocol_path.is_some()
        || descriptor.protocol_version.is_some()
    {
        let context_id =
            message
                .context_id
                .as_ref()
                .ok_or(HandleMessageError::InvalidDescriptor(
                    "No context id".to_string(),
                ))?;

        let protocol =
            descriptor
                .protocol
                .as_ref()
                .ok_or(HandleMessageError::InvalidDescriptor(
                    "No protocol".to_string(),
                ))?;

        let protocol_path =
            descriptor
                .protocol_path
                .as_ref()
                .ok_or(HandleMessageError::InvalidDescriptor(
                    "No protocol path".to_string(),
                ))?;

        let protocol_version =
            descriptor
                .protocol_version
                .as_ref()
                .ok_or(HandleMessageError::InvalidDescriptor(
                    "No protocol version".to_string(),
                ))?;

        // Get protocol from message store.
        let protocols = message_store
            .query_protocols(
                target.clone(),
                authorized,
                ProtocolsFilter {
                    protocol: protocol.clone(),
                    versions: vec![protocol_version.clone()],
                },
            )
            .await?;

        let protocol = protocols
            .first()
            .ok_or(HandleMessageError::InvalidDescriptor(
                "Protocol not found".to_string(),
            ))?;

        let protocol_descriptor = match &protocol.descriptor {
            Descriptor::ProtocolsConfigure(descriptor) => descriptor,
            _ => {
                return Err(HandleMessageError::InvalidDescriptor(
                    "Invalid protocol descriptor".to_string(),
                ))
            }
        };

        let definition = protocol_descriptor.definition.as_ref().ok_or(
            HandleMessageError::InvalidDescriptor("No protocol definition".to_string()),
        )?;

        // Ensure protocol is published if message is published.
        if !definition.published && descriptor.published.unwrap_or_default() {
            return Err(HandleMessageError::InvalidDescriptor(
                "Protocol not published".to_string(),
            ));
        }

        let structure = definition.get_structure(protocol_path).ok_or(
            HandleMessageError::InvalidDescriptor("Protocol structure not found".to_string()),
        )?;

        let structure_type = definition
            .types
            .get(protocol_path.split('/').last().unwrap())
            .ok_or(HandleMessageError::InvalidDescriptor(
                "Protocol type not found".to_string(),
            ))?;

        // Ensure data format matches.
        let data_format = match descriptor.data_format.clone() {
            Some(d) => Some(d),
            None => messages.iter().find_map(|m| match &m.descriptor {
                Descriptor::RecordsWrite(desc) => desc.data_format.clone(),
                _ => None,
            }),
        };

        if let Some(structure_format) = &structure_type.data_format {
            match &data_format {
                Some(data_format) => {
                    if !structure_format.contains(data_format) {
                        return Err(HandleMessageError::InvalidDescriptor(
                            "Data format does not match protocol definition".to_string(),
                        ));
                    }
                }
                None => {
                    return Err(HandleMessageError::InvalidDescriptor(
                        "Data format missing".to_string(),
                    ));
                }
            }
        }

        let mut context = Vec::new();

        let mut context_ids = context_id.split('/').collect::<Vec<_>>();
        context_ids.pop();

        let parents_path_len = protocol_path.split('/').count() - 1;

        if context_ids.len() != parents_path_len {
            return Err(HandleMessageError::InvalidDescriptor(
                "Context id does not match protocol path".to_string(),
            ));
        }

        for (i, record_id) in context_ids.iter().enumerate() {
            let messages = message_store
                .query_records(
                    target.clone(),
                    message.author(),
                    authorized,
                    RecordsFilter {
                        record_id: Some(record_id.to_string()),
                        date_sort: Some(FilterDateSort::CreatedDescending),
                        ..Default::default()
                    },
                )
                .await?;

            if messages.is_empty() {
                return Err(HandleMessageError::InvalidDescriptor(
                    "Context record not found".to_string(),
                ));
            }

            let message = &messages[0];

            // Validate that message matches the expected protocol path.
            let expected = protocol_path.split('/').nth(i).unwrap();

            let path = match &message.descriptor {
                Descriptor::RecordsWrite(desc) => desc.protocol_path.clone(),
                _ => None,
            };

            if path != Some(expected.to_string()) {
                return Err(HandleMessageError::InvalidDescriptor(
                    "Context record does not match protocol path".to_string(),
                ));
            }

            context.push(message.clone());
        }

        // Can only write root level protocol records by default.
        // If this is a child protocol sructure, we must meet the permission requirements.
        if !context.is_empty() {
            can_write = false;
        }

        let author = message.author();

        // Set write permissions.
        for action in &structure.actions {
            if !action.can.contains(&ActionCan::Write) {
                continue;
            }

            if action.who == ActionWho::Anyone {
                can_write = true;
                break;
            }

            let author = match author.as_deref() {
                Some(author) => author,
                None => continue,
            };

            if let Some(of) = &action.of {
                // Walk up context until we find 'of'.
                for context_msg in &context {
                    let context_path = match context_msg.descriptor {
                        Descriptor::RecordsWrite(ref desc) => desc.protocol_path.clone(),
                        _ => None,
                    };

                    let path = match context_path {
                        Some(path) => path,
                        None => continue,
                    };

                    if path == of.as_str() {
                        // Found 'of'.
                        can_write = match action.who {
                            ActionWho::Author => match context_msg.author().as_deref() {
                                Some(context_msg_author) => author == context_msg_author,
                                None => false,
                            },
                            ActionWho::Recipient => author == target,
                            ActionWho::Anyone => unreachable!(),
                        };

                        break;
                    }
                }
            }
        }
    }

    if !can_write {
        return Err(HandleMessageError::Unauthorized);
    }

    let entry_id = message.entry_id()?;

    if entry_id == message.record_id {
        if initial_entry.is_some() {
            // Initial entry already exists, cease processing.
            return Ok(StatusReply {
                status: Status::ok(),
            }
            .into());
        }

        // Store message as initial entry.
        message_store
            .put(authorized, target.clone(), message, data_store.clone())
            .await?;
    } else {
        let checkpoint_entry = checkpoint_entry.ok_or(HandleMessageError::InvalidDescriptor(
            "Checkpoint entry not found".to_string(),
        ))?;

        let checkpoint_entry_id = checkpoint_entry.entry_id()?;

        // If message is the checkpoint entry, cease processing.
        if checkpoint_entry_id == entry_id {
            return Ok(StatusReply {
                status: Status::ok(),
            }
            .into());
        }

        let parent_id =
            descriptor
                .parent_id
                .as_ref()
                .ok_or(HandleMessageError::InvalidDescriptor(
                    "No parent id".to_string(),
                ))?;

        // Ensure parent id matches the latest checkpoint entry.
        if !options.ignore_parent_id && *parent_id != checkpoint_entry_id {
            return Err(HandleMessageError::InvalidDescriptor(
                "Parent id does not match latest checkpoint entry".to_string(),
            ));
        }

        let checkpoint_time = match &checkpoint_entry.descriptor {
            Descriptor::RecordsDelete(desc) => desc.message_timestamp,
            Descriptor::RecordsWrite(desc) => desc.message_timestamp,
            _ => {
                return Err(HandleMessageError::InvalidDescriptor(
                    "Latest checkpoint is not a RecordsDelete or RecordsWrite message".to_string(),
                ))
            }
        };

        // Ensure message timestamp is greater than the latest checkpoint entry.
        if descriptor.message_timestamp <= checkpoint_time {
            return Err(HandleMessageError::InvalidDescriptor(format!(
                "Message timestamp ({}) is not greater than the latest checkpoint entry ({})",
                descriptor.message_timestamp, checkpoint_time
            )));
        }

        let existing_writes = messages
            .iter()
            .filter(|m| matches!(m.descriptor, Descriptor::RecordsWrite(_)))
            .filter(|m| m.record_id == message.record_id)
            .collect::<Vec<_>>();

        if existing_writes.is_empty() {
            // Store message as new entry.
            message_store
                .put(authorized, target.clone(), message, data_store.clone())
                .await?;
        } else if existing_writes.iter().all(|m| {
            let m_timestamp = match &m.descriptor {
                Descriptor::RecordsWrite(desc) => desc.message_timestamp,
                _ => unreachable!(),
            };

            // Ensure message timestamp is greater than the latest write.
            // If times are equal, ensure the entry id is greater.
            if descriptor.message_timestamp == m_timestamp {
                let m_entry_id = m.entry_id().unwrap();
                entry_id > m_entry_id
            } else {
                descriptor.message_timestamp > m_timestamp
            }
        }) {
            // Delete existing writes.
            for m in existing_writes {
                let cbor = encode_cbor(&m)?;
                message_store
                    .delete(target.clone(), cbor.cid().to_string(), data_store.clone())
                    .await?;
            }

            // Store message as new entry.
            message_store
                .put(authorized, target, message, data_store.clone())
                .await?;
        }
    }

    Ok(StatusReply {
        status: Status::ok(),
    }
    .into())
}
