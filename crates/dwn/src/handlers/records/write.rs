use crate::{
    handlers::{HandlerError, MethodHandler, Reply, Status, StatusReply},
    message::{
        descriptor::{Descriptor, Filter, FilterDateSort},
        Message,
    },
    store::{DataStore, MessageStore},
    util::encode_cbor,
};

pub struct RecordsWriteHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsWriteHandler<'_, D, M> {
    async fn handle(&self, message: Message) -> Result<impl Into<Reply>, HandlerError> {
        if message.authorization.is_none() {
            return Err(HandlerError::InvalidDescriptor(
                "No authorization".to_string(),
            ));
        }

        let tenant = message.tenant().await?.unwrap();

        let entry_id = message.generate_record_id()?;

        // Get messages for the record.
        let messages = self
            .message_store
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
                });
            }

            // Store message as initial entry.
            self.message_store
                .put(tenant.clone(), message, self.data_store)
                .await?;
        } else {
            let initial_entry = initial_entry.ok_or(HandlerError::InvalidDescriptor(
                "Initial entry not found".to_string(),
            ))?;

            let descriptor = match &message.descriptor {
                Descriptor::RecordsWrite(descriptor) => descriptor,
                _ => {
                    return Err(HandlerError::InvalidDescriptor(
                        "Not a RecordsWrite message".to_string(),
                    ))
                }
            };

            let parent_id = descriptor
                .parent_id
                .as_ref()
                .ok_or(HandlerError::InvalidDescriptor("No parent id".to_string()))?;

            // TODO: Ensure immutable values remain unchanged.

            let checkpoint_entry = messages
                .iter()
                .find(|m| matches!(m.descriptor, Descriptor::RecordsDelete(_)))
                .unwrap_or(initial_entry);

            let checkpoint_entry_id = checkpoint_entry.generate_record_id()?;

            // Ensure parent id matches the latest checkpoint entry.
            if *parent_id != checkpoint_entry_id {
                return Err(HandlerError::InvalidDescriptor(
                    "Parent id does not match latest checkpoint entry".to_string(),
                ));
            }

            let checkpoint_time = match &checkpoint_entry.descriptor {
                Descriptor::RecordsDelete(desc) => desc.message_timestamp,
                Descriptor::RecordsWrite(desc) => desc.message_timestamp,
                _ => {
                    return Err(HandlerError::InvalidDescriptor(
                        "Latest checkpoint is not a RecordsDelete or RecordsWrite message"
                            .to_string(),
                    ))
                }
            };

            // Ensure message timestamp is greater than the latest checkpoint entry.
            if descriptor.message_timestamp <= checkpoint_time {
                return Err(HandlerError::InvalidDescriptor(
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
                self.message_store
                    .put(tenant.clone(), message, self.data_store)
                    .await?;
            } else if existing_writes.iter().all(|m| {
                let m_timestamp = match &m.descriptor {
                    Descriptor::RecordsWrite(desc) => desc.message_timestamp,
                    _ => unreachable!(),
                };

                // Ensure message timestamp is greater than the latest write.
                // If times are equal, ensure the entry id is greater.
                if descriptor.message_timestamp == m_timestamp {
                    let m_entry_id = m.generate_record_id().unwrap();
                    entry_id > m_entry_id
                } else {
                    descriptor.message_timestamp > m_timestamp
                }
            }) {
                // Delete existing writes.
                for m in existing_writes {
                    let cbor = encode_cbor(&m)?;
                    self.message_store
                        .delete(&tenant, cbor.cid().to_string(), self.data_store)
                        .await?;
                }

                // Store message as new entry.
                self.message_store
                    .put(tenant, message, self.data_store)
                    .await?;
            }
        }

        Ok(StatusReply {
            status: Status::ok(),
        })
    }
}
