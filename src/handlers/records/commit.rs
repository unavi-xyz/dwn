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
            return Err(HandlerError::InvalidDescriptor(
                "Parent message does not descend from active message".to_string(),
            ));
        }

        let parent = match entry_id_to_msg.get(&descriptor.parent_id) {
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

#[cfg(test)]
mod tests {
    use crate::{
        message::{
            builder::MessageBuilder,
            data::Data,
            descriptor::{Descriptor, RecordsCommit, RecordsWrite},
        },
        tests::create_dwn,
        util::DidKey,
    };

    #[tokio::test]
    async fn require_auth() {
        let dwn = create_dwn().await;
        let did_key = DidKey::new().expect("Failed to generate DID key");

        // Fails without authorization.
        {
            let message = MessageBuilder::new::<RecordsCommit>()
                .build()
                .expect("Failed to build message");

            let reply = dwn.process_message(&did_key.did, message).await;
            assert!(reply.is_err());
        }

        // Succeeds with authorization.
        {
            // Write a record.
            let message1 = MessageBuilder::new::<RecordsWrite>()
                .authorize(did_key.kid.clone(), &did_key.jwk)
                .data(Data::Base64("Hello, world!".to_string()))
                .build()
                .expect("Failed to build message");

            let reply = dwn
                .process_message(&did_key.did, message1.clone())
                .await
                .expect("Failed to handle message");
            assert!(reply.status().code == 200);

            // Commit the record.
            let message2 = MessageBuilder::new::<RecordsCommit>()
                .authorize(did_key.kid, &did_key.jwk)
                .parent(&message1)
                .build()
                .expect("Failed to build message");

            let reply = dwn
                .process_message(&did_key.did, message2)
                .await
                .expect("Failed to handle message");
            assert!(reply.status().code == 200);
        }
    }

    #[tokio::test]
    async fn requires_valid_parent() {
        let dwn = create_dwn().await;
        let did_key = DidKey::new().expect("Failed to generate DID key");

        // Write a record.
        let message1 = MessageBuilder::new::<RecordsWrite>()
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .data(Data::Base64("Hello, world!".to_string()))
            .build()
            .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message1.clone())
            .await
            .expect("Failed to handle message");
        assert!(reply.status().code == 200);

        // Fails with missing parent.
        {
            let mut message2 = MessageBuilder::new::<RecordsCommit>()
                .authorize(did_key.kid.clone(), &did_key.jwk)
                .build()
                .expect("Failed to build message");

            match &mut message2.descriptor {
                Descriptor::RecordsCommit(desc) => {
                    desc.parent_id = "missing".to_string();
                }
                _ => panic!("Unexpected descriptor"),
            }

            let reply = dwn.process_message(&did_key.did, message2).await;
            assert!(reply.is_err());
        }

        // Fails with parent for different record.
        {
            let message2 = MessageBuilder::new::<RecordsWrite>()
                .authorize(did_key.kid.clone(), &did_key.jwk)
                .data(Data::Base64("Goodbye, world!".to_string()))
                .build()
                .expect("Failed to build message");

            let mut message3 = MessageBuilder::new::<RecordsCommit>()
                .authorize(did_key.kid, &did_key.jwk)
                .parent(&message2)
                .build()
                .expect("Failed to build message");

            message3.record_id = message1.record_id.clone();

            let reply = dwn
                .process_message(&did_key.did, message2)
                .await
                .expect("Failed to handle message");
            assert!(reply.status().code == 200);

            let reply = dwn.process_message(&did_key.did, message3).await;
            assert!(reply.is_err());
        }
    }
}
