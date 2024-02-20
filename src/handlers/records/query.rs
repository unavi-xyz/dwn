use crate::{
    handlers::{HandlerError, MethodHandler, Reply, Status},
    message::{descriptor::Descriptor, Message},
    store::{DataStore, MessageStore},
};

pub struct RecordsQueryHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsQueryHandler<'_, D, M> {
    async fn handle(&self, tenant: &str, message: Message) -> Result<Reply, HandlerError> {
        let filter = match message.descriptor {
            Descriptor::RecordsQuery(descriptor) => descriptor.filter,
            _ => {
                return Err(HandlerError::InvalidDescriptor(
                    "Not a RecordsQuery message".to_string(),
                ))
            }
        };

        let entries = self
            .message_store
            .query(tenant, filter.unwrap_or_default())
            .await?;

        Ok(Reply::RecordsQuery {
            entries,
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
            descriptor::{Filter, RecordsQuery, RecordsWrite},
        },
        tests::create_dwn,
        util::DidKey,
    };

    #[tokio::test]
    async fn query_by_record_id() {
        let dwn = create_dwn().await;
        let did_key = DidKey::new().expect("Failed to generate DID key");

        // Write some records.
        let message1 = MessageBuilder::new(RecordsWrite::default())
            .data(Data::Base64("Hello, world!".to_string()))
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .build()
            .expect("Failed to build message");

        let message2 = MessageBuilder::new(RecordsWrite::default())
            .data(Data::Base64("Goodbye, world!".to_string()))
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .build()
            .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message1.clone())
            .await
            .expect("Failed to process message");

        assert_eq!(reply.status().code, 200);

        let reply = dwn
            .process_message(&did_key.did, message2.clone())
            .await
            .expect("Failed to process message");

        assert_eq!(reply.status().code, 200);

        // Query the record id.
        let message3 = MessageBuilder::new(RecordsQuery::new(Filter {
            record_id: Some(message1.record_id.clone()),
            ..Default::default()
        }))
        .build()
        .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message3)
            .await
            .expect("Failed to process message");

        assert_eq!(reply.status().code, 200);

        let entries = match reply {
            Reply::RecordsQuery { entries, .. } => entries,
            _ => panic!("Unexpected reply"),
        };

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].record_id, message1.record_id);

        // Query the other record id.
        let message4 = MessageBuilder::new(RecordsQuery::new(Filter {
            record_id: Some(message2.record_id.clone()),
            ..Default::default()
        }))
        .build()
        .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message4)
            .await
            .expect("Failed to process message");

        assert_eq!(reply.status().code, 200);

        let entries = match reply {
            Reply::RecordsQuery { entries, .. } => entries,
            _ => panic!("Unexpected reply"),
        };

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].record_id, message2.record_id);
    }
}
