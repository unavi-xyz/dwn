use crate::{
    handlers::{HandlerError, MethodHandler, RecordsReadReply, Reply, Status},
    message::{
        descriptor::{Descriptor, Filter, FilterDateSort},
        Message,
    },
    store::{DataStore, MessageStore},
};

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

        // TODO: Get data from data store.

        Ok(RecordsReadReply {
            data: Vec::new(),
            record: record.clone(),
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
            data::Data,
            descriptor::{RecordsCommit, RecordsRead, RecordsWrite},
        },
        tests::create_dwn,
        util::DidKey,
    };

    #[tokio::test]
    async fn read_initial_entry() {
        let dwn = create_dwn().await;
        let did_key = DidKey::new().expect("Failed to generate DID key");

        // Create a record.
        let message1 = MessageBuilder::new::<RecordsWrite>()
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .data(Data::Base64("Hello, world!".to_string()))
            .build()
            .expect("Failed to build message");

        let record_id = message1.record_id.clone();

        dwn.process_message(&did_key.did, message1.clone())
            .await
            .expect("Failed to handle message");

        // Read the record.
        let message2 = MessageBuilder::from_descriptor(RecordsRead::new(record_id.clone()))
            .build()
            .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message2)
            .await
            .expect("Failed to handle message");

        match reply {
            Reply::RecordsRead(reply) => {
                let mut message1_stripped = message1.clone();
                message1_stripped.data = None;

                assert_eq!(reply.status.code, 200);
                assert_eq!(reply.record, message1_stripped);

                // TODO: Check data.
            }
            _ => panic!("Unexpected reply: {:?}", reply),
        }
    }

    #[tokio::test]
    async fn read_commit() {
        let dwn = create_dwn().await;
        let did_key = DidKey::new().expect("Failed to generate DID key");

        // Create a record.
        let message1 = MessageBuilder::new::<RecordsWrite>()
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .data(Data::Base64("Hello, world!".to_string()))
            .build()
            .expect("Failed to build message");

        dwn.process_message(&did_key.did, message1.clone())
            .await
            .expect("Failed to handle message");

        // Update the record.
        let message2 = MessageBuilder::new::<RecordsWrite>()
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .data(Data::Base64("Goodbye, world!".to_string()))
            .parent(&message1)
            .build()
            .expect("Failed to build message");

        let message3 = MessageBuilder::new::<RecordsCommit>()
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .parent(&message2)
            .build()
            .expect("Failed to build message");

        dwn.process_message(&did_key.did, message2)
            .await
            .expect("Failed to handle message");

        dwn.process_message(&did_key.did, message3.clone())
            .await
            .expect("Failed to handle message");

        // Read the record.
        let message4 = MessageBuilder::from_descriptor(RecordsRead::new(message1.record_id))
            .build()
            .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message4)
            .await
            .expect("Failed to handle message");

        match reply {
            Reply::RecordsRead(reply) => {
                assert_eq!(reply.status.code, 200);
                assert_eq!(reply.record, message3);

                // TODO: Check data.
            }
            _ => panic!("Unexpected reply: {:?}", reply),
        }
    }
}
