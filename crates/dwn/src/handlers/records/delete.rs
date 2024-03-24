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

pub async fn handle_records_delete(
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

    let descriptor = match &message.descriptor {
        Descriptor::RecordsDelete(desc) => desc,
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a RecordsDelete message".to_string(),
            ));
        }
    };

    // TODO: Ensure all immutable values from inital entry are not changed.

    let messages = message_store
        .query(
            Some(tenant.clone()),
            Filter {
                record_id: Some(descriptor.record_id.clone()),
                date_sort: Some(FilterDateSort::CreatedDescending),
                ..Default::default()
            },
        )
        .await?;

    let active = messages
        .iter()
        .find(|m| matches!(m.descriptor, Descriptor::RecordsDelete(_)));

    if let Some(active) = active {
        let active_desc = match &active.descriptor {
            Descriptor::RecordsDelete(desc) => desc,
            _ => {
                return Err(HandleMessageError::InvalidDescriptor(
                    "Active delete message not a RecordsDelete message?".to_string(),
                ));
            }
        };

        // If the active delete message is newer, cease processing.
        if descriptor.message_timestamp < active_desc.message_timestamp {
            return Ok(StatusReply {
                status: Status::ok(),
            }
            .into());
        }
    }

    // Delete all messages for the record.
    for m in messages {
        let block = encode_cbor(&m)?;
        message_store
            .delete(&tenant, block.cid().to_string(), data_store)
            .await?;
    }

    // Store the message.
    message_store.put(tenant, message, data_store).await?;

    Ok(StatusReply {
        status: Status::ok(),
    }
    .into())
}
