use handlers::{records::write::RecordsWriteHandler, MethodHandler};
use message::descriptor::Descriptor;
use store::{surrealdb::message::MessageStoreError, DataStore, MessageStore};
use thiserror::Error;

pub mod handlers;
pub mod message;
pub mod store;

pub struct DWN<D: DataStore, M: MessageStore> {
    pub data_store: D,
    pub message_store: M,
}

#[derive(Error, Debug)]
pub enum HandleMessageError {
    #[error("Message store error: {0}")]
    MessageStoreError(#[from] MessageStoreError),
    #[error("Unsupported interface")]
    UnsupportedInterface,
}

impl<D: DataStore, M: MessageStore> DWN<D, M> {
    pub fn handle_message(
        &self,
        tenant: &str,
        message: message::Message,
    ) -> Result<handlers::MessageReply, HandleMessageError> {
        match &message.descriptor {
            Descriptor::RecordsWrite(_) => {
                let handler = RecordsWriteHandler {
                    data_store: &self.data_store,
                    message_store: &self.message_store,
                };
                let reply = handler.handle(tenant, message)?;
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
            descriptor::{records::RecordsWrite, Descriptor},
            Message,
        },
        store::surrealdb::SurrealDB,
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
            record_id: None,
        };

        let tenant = "did:example:123";

        let reply = dwn
            .handle_message(tenant, message)
            .expect("Failed to handle message");

        assert_eq!(reply.status.code, 200);
    }
}
