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
            descriptor::{Descriptor, RecordsWrite},
            Message,
        },
        store::SurrealDB,
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

        let message = Message {
            attestation: None,
            authorization: None,
            data: None,
            descriptor: Descriptor::RecordsWrite(RecordsWrite::default()),
            record_id: "".to_string(),
        };

        let tenant = "did:example:123";

        // Require authorization
        {
            let reply = dwn.process_message(tenant, message).await;
            assert!(reply.is_err());
        }
    }
}
