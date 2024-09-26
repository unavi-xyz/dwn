use std::sync::Arc;

use crate::{
    encode::encode_cbor,
    message::{
        descriptor::{
            records::{FilterDateSort, RecordsFilter},
            Descriptor,
        },
        DwnRequest,
    },
    reply::{MessageReply, Status, StatusReply},
    store::{DataStore, MessageStore},
    HandleMessageError,
};

pub async fn handle_records_delete(
    data_store: Arc<dyn DataStore>,
    message_store: &Arc<dyn MessageStore>,
    DwnRequest { target, message }: DwnRequest,
) -> Result<MessageReply, HandleMessageError> {
    let authorized = message.is_authorized(&target).await;

    if !authorized {
        return Err(HandleMessageError::Unauthorized);
    }

    let descriptor = match &message.descriptor {
        Descriptor::RecordsDelete(desc) => desc,
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a RecordsDelete message".to_string(),
            ));
        }
    };

    let messages = message_store
        .query_records(
            target.clone(),
            message.author(),
            authorized,
            RecordsFilter {
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
    for m in messages.iter() {
        let block = encode_cbor(m)?;
        message_store
            .delete(target.clone(), block.cid().to_string(), data_store.clone())
            .await?;
    }

    // Store the delete message.
    message_store
        .put(authorized, target, message, data_store)
        .await?;

    Ok(StatusReply {
        status: Status::ok(),
    }
    .into())
}
