use crate::{
    handlers::{HandlerError, MethodHandler, Reply, Status, StatusReply},
    message::{
        descriptor::{Descriptor, Filter, FilterDateSort},
        Message,
    },
    store::{DataStore, MessageStore},
    util::encode_cbor,
};

pub struct RecordsDeleteHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsDeleteHandler<'_, D, M> {
    async fn handle(&self, message: Message) -> Result<impl Into<Reply>, HandlerError> {
        if message.attestation.is_none() {
            return Err(HandlerError::InvalidDescriptor(
                "No attestation".to_string(),
            ));
        }

        if message.authorization.is_none() {
            return Err(HandlerError::InvalidDescriptor(
                "No authorization".to_string(),
            ));
        }

        let tenant = message.verify_attestation().await.unwrap()[0].to_string();

        let descriptor = match &message.descriptor {
            Descriptor::RecordsDelete(desc) => desc,
            _ => {
                return Err(HandlerError::InvalidDescriptor(
                    "Not a RecordsDelete message".to_string(),
                ));
            }
        };

        // TODO: Ensure all immutable values from inital entry are not changed.

        let messages = self
            .message_store
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
                    return Err(HandlerError::InvalidDescriptor(
                        "Active delete message not a RecordsDelete message?".to_string(),
                    ));
                }
            };

            // If the active delete message is newer, cease processing.
            if descriptor.message_timestamp < active_desc.message_timestamp {
                return Ok(StatusReply {
                    status: Status::ok(),
                });
            }
        }

        // Delete all messages for the record.
        for m in messages {
            let block = encode_cbor(&m)?;
            self.message_store
                .delete(&tenant, block.cid().to_string(), self.data_store)
                .await?;
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
