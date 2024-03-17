use std::collections::HashMap;

use crate::{
    handlers::{HandlerError, MethodHandler, Reply, Status, StatusReply},
    message::{
        descriptor::{Descriptor, Filter, FilterDateSort},
        Message,
    },
    store::{DataStore, MessageStore},
};

use super::util::create_entry_id_map;

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
        message.verify_auth().await?;

        let descriptor = match &message.descriptor {
            Descriptor::RecordsCommit(desc) => desc,
            _ => {
                return Err(HandlerError::InvalidDescriptor(
                    "Not a RecordsCommit message".to_string(),
                ));
            }
        };

        // Get currently active RecordsWrite message.
        let messages = self
            .message_store
            .query(
                tenant,
                Filter {
                    record_id: Some(message.record_id.clone()),
                    date_sort: Some(FilterDateSort::CreatedDescending),
                    ..Default::default()
                },
            )
            .await?;

        let active = messages
            .iter()
            .find(|m| matches!(m.descriptor, Descriptor::RecordsWrite(_)))
            .ok_or(HandlerError::InvalidDescriptor(
                "No active RecordsWrite message found for record".to_string(),
            ))?;

        // TODO: Ensure immutable values from inital entry are not changed.

        // Parent id must match either the active message, or another RecordsCommit that descends from it.
        let active_entry_id = active.generate_record_id()?;
        let entry_id_map = create_entry_id_map(&messages)?;

        if !descends_from(&descriptor.parent_id, &active_entry_id, &entry_id_map) {
            return Err(HandlerError::InvalidDescriptor(
                "Parent message does not descend from active message".to_string(),
            ));
        }

        let parent = match entry_id_map.get(&descriptor.parent_id) {
            Some(m) => m,
            None => {
                return Err(HandlerError::InvalidDescriptor(
                    "Parent message not found".to_string(),
                ));
            }
        };

        let parent_timestamp = match &parent.descriptor {
            Descriptor::RecordsCommit(desc) => desc.message_timestamp,
            Descriptor::RecordsWrite(desc) => desc.message_timestamp,
            _ => {
                return Err(HandlerError::InvalidDescriptor(
                    "Parent message is not a commit or write".to_string(),
                ))
            }
        };

        // Ensure message is not older than parent.
        if descriptor.message_timestamp < parent_timestamp {
            return Err(HandlerError::InvalidDescriptor(
                "Message timestamp is older than parent".to_string(),
            ));
        }

        // Store the message.
        self.message_store
            .put(tenant, message, self.data_store)
            .await?;

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
