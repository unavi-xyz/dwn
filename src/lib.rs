use handlers::{records::write::RecordsWriteHandler, HandlerError, MessageReply, MethodHandler};
use message::{descriptor::Descriptor, Message};
use store::{DataStore, MessageStore};
use thiserror::Error;

pub mod handlers;
pub mod message;
pub mod store;
pub mod util;

pub struct DWN<D: DataStore, M: MessageStore> {
    pub data_store: D,
    pub message_store: M,
}

#[derive(Error, Debug)]
pub enum HandleMessageError {
    #[error("Unsupported interface")]
    UnsupportedInterface,
    #[error("Failed to handle message: {0}")]
    HandlerError(#[from] HandlerError),
}

impl<D: DataStore, M: MessageStore> DWN<D, M> {
    pub async fn process_message(
        &self,
        tenant: &str,
        message: Message,
    ) -> Result<MessageReply, HandleMessageError> {
        match &message.descriptor {
            Descriptor::RecordsWrite(_) => {
                let handler = RecordsWriteHandler {
                    data_store: &self.data_store,
                    message_store: &self.message_store,
                };
                let reply = handler.handle(tenant, message).await?;
                Ok(reply)
            }
            _ => Err(HandleMessageError::UnsupportedInterface),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        message::{
            builder::MessageBuilder,
            descriptor::{RecordsWrite},
        },
        store::SurrealDB,
        util::DidKey,
        DWN,
    };

    async fn create_dwn() -> DWN<SurrealDB, SurrealDB> {
        let db = SurrealDB::new().await.expect("Failed to create SurrealDB");
        DWN {
            data_store: db.clone(),
            message_store: db,
        }
    }

    #[tokio::test]
    async fn test_records_write() {
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
}
