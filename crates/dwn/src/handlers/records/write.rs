use crate::{
    encode::encode_cbor,
    handlers::{MessageReply, Status, StatusReply},
    message::{
        descriptor::{
            protocols::{ActionCan, ActionWho, ProtocolsFilter},
            records::{FilterDateSort, RecordsFilter},
            Descriptor,
        },
        DwnRequest,
    },
    store::{DataStore, MessageStore},
    HandleMessageError,
};

pub async fn handle_records_write(
    data_store: &impl DataStore,
    message_store: &impl MessageStore,
    DwnRequest { target, message }: DwnRequest,
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

    // Get messages for the record.
    let messages = message_store
        .query_records(
            target.clone(),
            None,
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
    }

    // Validate protocol rules.
    let mut can_write = authorized;

    if message.context_id.is_some()
        || descriptor.protocol.is_some()
        || descriptor.protocol_path.is_some()
        || descriptor.protocol_version.is_some()
    {
        let _context_id =
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

        // Get structure from definition.
        let structure = definition.structure.get(protocol_path).ok_or(
            HandleMessageError::InvalidDescriptor("Protocol path not found".to_string()),
        )?;

        // Get type from definition.
        let structure_type =
            definition
                .types
                .get(protocol_path)
                .ok_or(HandleMessageError::InvalidDescriptor(
                    "Protocol type not found".to_string(),
                ))?;

        // Ensure data format matches.
        let data_format = match descriptor.data_format.clone() {
            Some(data_format) => Some(data_format),
            None => messages.iter().find_map(|m| match &m.descriptor {
                Descriptor::RecordsWrite(desc) => desc.data_format.clone(),
                _ => None,
            }),
        };

        if !structure_type.data_format.is_empty() {
            if let Some(data_format) = data_format {
                if !structure_type.data_format.contains(&data_format) {
                    return Err(HandleMessageError::InvalidDescriptor(
                        "Data format does not match protocol type".to_string(),
                    ));
                }
            } else {
                return Err(HandleMessageError::InvalidDescriptor(
                    "No data format".to_string(),
                ));
            }
        }

        // Ensure write permissions.

        // TODO: Support 'of'
        // TODO: Validate context id

        if !can_write {
            for action in &structure.actions {
                if action.can != ActionCan::Write {
                    continue;
                }

                if action.who == ActionWho::Anyone {
                    can_write = true;
                    break;
                }
            }
        };
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
            .put(target.clone(), message, data_store)
            .await?;
    } else {
        let checkpoint_entry = checkpoint_entry.ok_or(HandleMessageError::InvalidDescriptor(
            "Checkpoint entry not found".to_string(),
        ))?;

        let parent_id =
            descriptor
                .parent_id
                .as_ref()
                .ok_or(HandleMessageError::InvalidDescriptor(
                    "No parent id".to_string(),
                ))?;

        // TODO: Ensure immutable values remain unchanged.

        let checkpoint_entry_id = checkpoint_entry.entry_id()?;

        // Ensure parent id matches the latest checkpoint entry.
        if *parent_id != checkpoint_entry_id {
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
            return Err(HandleMessageError::InvalidDescriptor(
                "Message timestamp is not greater than the latest checkpoint entry".to_string(),
            ));
        }

        let existing_writes = messages
            .iter()
            .filter(|m| matches!(m.descriptor, Descriptor::RecordsWrite(_)))
            .filter(|m| m.record_id == message.record_id)
            .collect::<Vec<_>>();

        if existing_writes.is_empty() {
            // Store message as new entry.
            message_store
                .put(target.clone(), message, data_store)
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
                    .delete(&target, cbor.cid().to_string(), data_store)
                    .await?;
            }

            // Store message as new entry.
            message_store.put(target, message, data_store).await?;
        }
    }

    Ok(StatusReply {
        status: Status::ok(),
    }
    .into())
}
