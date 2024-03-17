use std::collections::HashMap;

use libipld::Cid;

use crate::{
    handlers::{HandlerError, MethodHandler, RecordsReadReply, Reply, Status},
    message::{
        descriptor::{Descriptor, Filter, FilterDateSort},
        Message,
    },
    store::{DataStore, MessageStore},
};

use super::util::create_entry_id_map;

pub struct RecordsReadHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsReadHandler<'_, D, M> {
    async fn handle(
        &self,
        tenant: &str,
        message: Message,
    ) -> Result<impl Into<Reply>, HandlerError> {
        let descriptor = match &message.descriptor {
            Descriptor::RecordsRead(descriptor) => descriptor,
            _ => {
                return Err(HandlerError::InvalidDescriptor(
                    "Not a RecordsRead message".to_string(),
                ));
            }
        };

        let messages = self
            .message_store
            .query(
                tenant,
                Filter {
                    record_id: Some(descriptor.record_id.clone()),
                    date_sort: Some(FilterDateSort::CreatedDescending),
                    ..Default::default()
                },
            )
            .await?;

        // Get the latest commit or delete message.
        let latest_commit_or_delete = messages.iter().find(|m| {
            matches!(
                m.descriptor,
                Descriptor::RecordsCommit(_) | Descriptor::RecordsDelete(_)
            )
        });

        // If no message was found, use the initial entry.
        let record =
            latest_commit_or_delete
                .or(messages.last())
                .ok_or(HandlerError::InvalidDescriptor(
                    "Record not found".to_string(),
                ))?;

        // Get the record that has the data.
        let entry_id_map = create_entry_id_map(&messages)?;
        let record_entry_id = record.generate_record_id()?;
        let data_cid = get_data_cid(&record_entry_id, &entry_id_map);

        let data = match data_cid {
            Some(data_cid) => {
                let data_cid = Cid::try_from(data_cid).map_err(|e| {
                    HandlerError::InvalidDescriptor(format!("Invalid data CID: {}", e))
                })?;
                let res = self.data_store.get(data_cid.to_string()).await?;
                res.map(|res| res.data)
            }
            None => None,
        };

        Ok(RecordsReadReply {
            data,
            record: record.to_owned(),
            status: Status::ok(),
        })
    }
}

/// Get the data CID for a given entry ID.
/// This will search up the chain of parent messages until it finds a RecordsWrite message.
fn get_data_cid(entry_id: &str, messages: &HashMap<String, &Message>) -> Option<String> {
    let entry = messages.get(entry_id)?;

    match &entry.descriptor {
        Descriptor::RecordsCommit(desc) => get_data_cid(&desc.parent_id, messages),
        Descriptor::RecordsWrite(desc) => desc.data_cid.clone(),
        _ => None,
    }
}
