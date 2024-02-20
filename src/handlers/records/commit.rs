use std::collections::HashMap;

use crate::{
    handlers::{HandlerError, MethodHandler, Reply, Status, StatusReply},
    message::{
        descriptor::{Descriptor, Filter, FilterDateSort},
        Message,
    },
    store::{DataStore, MessageStore},
};

pub struct RecordsCommitHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsCommitHandler<'_, D, M> {
    async fn handle(
        &self,
        tenant: &str,
        message: Message,
    ) -> Result<impl Into<Reply>, HandlerError> {
        let descriptor = match &message.descriptor {
            Descriptor::RecordsCommit(desc) => desc,
            _ => return Err(HandlerError::InvalidDescriptor),
        };

        // Get currently active RecordsWrite message.
        let messages = self
            .message_store
            .query(
                tenant,
                Filter {
                    record_id: Some(message.record_id.clone()),
                    date_sort: Some(FilterDateSort::PublishedDescending),
                    ..Default::default()
                },
            )
            .await?;

        let active = messages
            .iter()
            .find(|m| matches!(m.descriptor, Descriptor::RecordsWrite(_)))
            .ok_or(HandlerError::InvalidDescriptor)?;

        // TODO: Ensure immutable values from inital entry are not changed.

        let active_entry_id = active.generate_record_id()?;

        let entry_id_to_msg =
            messages
                .iter()
                .try_fold(HashMap::new(), |mut acc, m| -> Result<_, HandlerError> {
                    let entry_id = m.generate_record_id()?;
                    acc.insert(entry_id, m);
                    Ok(acc)
                })?;

        // Parent id must match either the active message, or another RecordsCommit that descends from it.
        if !descends_from(&descriptor.parent_id, &active_entry_id, &entry_id_to_msg) {
            return Err(HandlerError::InvalidDescriptor);
        }

        let parent = match entry_id_to_msg.get(&descriptor.parent_id) {
            Some(m) => m,
            None => return Err(HandlerError::InvalidDescriptor),
        };

        let parent_timestamp = match &parent.descriptor {
            Descriptor::RecordsCommit(desc) => desc.message_timestamp,
            Descriptor::RecordsWrite(desc) => desc.message_timestamp,
            _ => return Err(HandlerError::InvalidDescriptor),
        };

        // Ensure message is not older than parent.
        if descriptor.message_timestamp < parent_timestamp {
            return Err(HandlerError::InvalidDescriptor);
        }

        // Store the message.
        self.message_store.put(tenant, message).await?;

        Ok(StatusReply {
            status: Status::ok(),
        })
    }
}

/// Does RecordsCommit message `entry_id` descend from message `root_entry_id`?
fn descends_from(
    entry_id: &str,
    root_entry_id: &str,
    messages: &HashMap<String, &Message>,
) -> bool {
    if entry_id == root_entry_id {
        return true;
    }

    let message = match messages.get(entry_id) {
        Some(m) => m,
        None => return false,
    };

    let descriptor = match &message.descriptor {
        Descriptor::RecordsCommit(desc) => desc,
        _ => return false,
    };

    descends_from(&descriptor.parent_id, root_entry_id, messages)
}
