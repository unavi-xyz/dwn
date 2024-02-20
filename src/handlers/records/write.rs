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
    async fn handle(
        &self,
        tenant: &str,
        message: Message,
    ) -> Result<impl Into<Reply>, HandlerError> {
        message.verify_auth().await?;

        let entry_id = message.generate_record_id()?;

        // Get messages for the record.
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

        let initial_entry = messages.last();

        if entry_id == message.record_id {
            if initial_entry.is_some() {
                // Initial entry already exists, cease processing.
                return Ok(StatusReply {
                    status: Status::ok(),
                });
            } else {
                // Store message as initial entry.
                self.message_store.put(tenant, message).await?;
            }
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

            let latest_checkpoint_entry = messages
                .iter()
                .find(|m| matches!(m.descriptor, Descriptor::RecordsDelete(_)))
                .unwrap_or(initial_entry);

            let checkpoint_entry_id = latest_checkpoint_entry.generate_record_id()?;

            // Ensure parent id matches the latest checkpoint entry.
            if *parent_id != checkpoint_entry_id {
                return Err(HandlerError::InvalidDescriptor(
                    "Parent id does not match latest checkpoint entry".to_string(),
                ));
            }

            let checkpoint_time = match &latest_checkpoint_entry.descriptor {
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
                self.message_store.put(tenant, message).await?;
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
                        .delete(tenant, cbor.cid().to_string())
                        .await?;
                }

                // Store message as new entry.
                self.message_store.put(tenant, message).await?;
            } else {
                // Cease processing.
                return Ok(StatusReply {
                    status: Status::ok(),
                });
            }
        }

        // TODO: Store data

        Ok(StatusReply {
            status: Status::ok(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        handlers::Reply,
        message::{
            builder::MessageBuilder,
            descriptor::{Descriptor, Filter, RecordsQuery, RecordsWrite},
        },
        tests::create_dwn,
        util::DidKey,
    };

    #[tokio::test]
    async fn require_auth() {
        let dwn = create_dwn().await;
        let did_key = DidKey::new().expect("Failed to generate DID key");

        // Fails without authorization
        {
            let message = MessageBuilder::new(RecordsWrite::default())
                .build()
                .expect("Failed to build message");

            let reply = dwn.process_message(&did_key.did, message).await;
            assert!(reply.is_err());
        }

        // Succeeds with authorization
        {
            let message = MessageBuilder::new(RecordsWrite::default())
                .authorize(did_key.kid, &did_key.jwk)
                .build()
                .expect("Failed to build message");

            let reply = dwn.process_message(&did_key.did, message).await;
            assert!(reply.is_ok());
        }
    }

    #[tokio::test]
    async fn initial_entry() {
        let dwn = create_dwn().await;
        let did_key = DidKey::new().expect("Failed to generate DID key");

        // Create initial entry
        {
            let message = MessageBuilder::new(RecordsWrite::default())
                .authorize(did_key.kid.clone(), &did_key.jwk)
                .build()
                .expect("Failed to build message");

            let reply = dwn.process_message(&did_key.did, message).await;
            assert!(reply.is_ok());
        }

        // Succeeds with same entry, but doesn't write
        {
            let message1 = MessageBuilder::new(RecordsWrite::default())
                .authorize(did_key.kid.clone(), &did_key.jwk)
                .build()
                .expect("Failed to build message");

            let reply = dwn.process_message(&did_key.did, message1.clone()).await;
            assert!(reply.is_ok());

            // Ensure only initial entry exists
            let mut query = RecordsQuery::default();
            query.filter = Some(Filter {
                record_id: Some(message1.record_id.clone()),
                ..Default::default()
            });

            let message2 = MessageBuilder::new(query)
                .authorize(did_key.kid, &did_key.jwk)
                .build()
                .expect("Failed to build message");

            let messages = dwn.process_message(&did_key.did, message2).await;
            assert!(messages.is_ok());

            let reply = match messages.unwrap() {
                Reply::RecordsQuery(reply) => reply,
                _ => panic!("Unexpected reply"),
            };

            assert_eq!(reply.entries.len(), 1);

            let entry = &reply.entries[0];
            let entry_descriptor = match &entry.descriptor {
                Descriptor::RecordsWrite(desc) => desc,
                _ => panic!("Unexpected descriptor"),
            };

            let descriptor1 = match &message1.descriptor {
                Descriptor::RecordsWrite(desc) => desc,
                _ => panic!("Unexpected descriptor"),
            };

            assert_eq!(
                entry_descriptor.message_timestamp,
                descriptor1.message_timestamp
            );
        }
    }
}
