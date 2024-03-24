use crate::{
    handlers::{Reply, Status, StatusReply},
    message::{
        descriptor::{Descriptor, Filter, FilterDateSort},
        Message,
    },
    store::{DataStore, MessageStore},
    util::encode_cbor,
    HandleMessageError,
};

pub async fn handle_records_write(
    data_store: &impl DataStore,
    message_store: &impl MessageStore,
    message: Message,
) -> Result<Reply, HandleMessageError> {
    if message.authorization.is_none() {
        return Err(HandleMessageError::Unauthorized);
    }

    let tenant = match message.tenant() {
        Some(tenant) => tenant,
        None => return Err(HandleMessageError::Unauthorized),
    };

    let entry_id = message.entry_id()?;

    // Get messages for the record.
    let messages = message_store
        .query(
            Some(tenant.clone()),
            Filter {
                record_id: Some(message.record_id.clone()),
                date_sort: Some(FilterDateSort::CreatedDescending),
                ..Default::default()
            },
        )
        .await?;

    let initial_entry = messages.last();

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
            .put(tenant.clone(), message, data_store)
            .await?;
    } else {
        let initial_entry = initial_entry.ok_or(HandleMessageError::InvalidDescriptor(
            "Initial entry not found".to_string(),
        ))?;

        let descriptor = match &message.descriptor {
            Descriptor::RecordsWrite(descriptor) => descriptor,
            _ => {
                return Err(HandleMessageError::InvalidDescriptor(
                    "Not a RecordsWrite message".to_string(),
                ))
            }
        };

        let parent_id =
            descriptor
                .parent_id
                .as_ref()
                .ok_or(HandleMessageError::InvalidDescriptor(
                    "No parent id".to_string(),
                ))?;

        // TODO: Ensure immutable values remain unchanged.

        let checkpoint_entry = messages
            .iter()
            .find(|m| matches!(m.descriptor, Descriptor::RecordsDelete(_)))
            .unwrap_or(initial_entry);

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
                .put(tenant.clone(), message, data_store)
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
                    .delete(&tenant, cbor.cid().to_string(), data_store)
                    .await?;
            }

            // Store message as new entry.
            message_store.put(tenant, message, data_store).await?;
        }
    }

    Ok(StatusReply {
        status: Status::ok(),
    }
    .into())
}
