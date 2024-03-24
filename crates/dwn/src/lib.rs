//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! ## Usage
//!
//! ```
//! use dwn::{actor::{Actor, CreateRecord}, store::SurrealDB, DWN};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a DWN, using an embedded SurrealDB for storage.
//!     let db = SurrealDB::new().await.unwrap();
//!     let dwn = DWN::new(db);
//!
//!     // Create an actor to send messages.
//!     // Here we generate a new `did:key` for the actor's identity,
//!     // but you could use any DID method.
//!     let actor = Actor::new_did_key(dwn).unwrap();
//!
//!     // Create a new record.
//!     let data = "Hello, world!".bytes().collect::<Vec<_>>();
//!
//!     let create = actor
//!         .create(CreateRecord {
//!             data: Some(data.clone()),
//!             ..Default::default()
//!         })
//!         .await
//!         .unwrap();
//!
//!     assert_eq!(create.reply.status.code, 200);
//!
//!     // Read the record.
//!     let read = actor.read(create.record_id).await.unwrap();
//!
//!     assert_eq!(read.status.code, 200);
//!     assert_eq!(read.data, Some(data));
//! }
//! ```

use handlers::{
    records::{
        delete::handle_records_delete, query::handle_records_query, read::handle_records_read,
        write::handle_records_write,
    },
    Reply, Response, Status,
};
use message::{
    descriptor::Descriptor, Message, RawMessage, Request, ValidateError, ValidatedMessage,
};
use store::{DataStore, DataStoreError, MessageStore, MessageStoreError};
use thiserror::Error;

pub mod actor;
pub mod handlers;
pub mod message;
pub mod store;
pub mod util;

use tracing::warn;
use util::EncodeError;

use crate::handlers::StatusReply;

#[derive(Clone)]
pub struct DWN<D: DataStore, M: MessageStore> {
    pub data_store: D,
    pub message_store: M,
}

#[derive(Debug, Error)]
pub enum HandleMessageError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Unsupported interface")]
    UnsupportedInterface,
    #[error(transparent)]
    ValidateError(#[from] ValidateError),
    #[error("Invalid descriptor")]
    InvalidDescriptor(String),
    #[error(transparent)]
    DataStoreError(#[from] DataStoreError),
    #[error(transparent)]
    MessageStoreError(#[from] MessageStoreError),
    #[error(transparent)]
    CborEncode(#[from] EncodeError),
}

impl<D: DataStore, M: MessageStore> DWN<D, M> {
    pub async fn process_request(&self, payload: Request) -> Response {
        let mut replies = Vec::new();

        for message in payload.messages {
            match self.process_message(message).await {
                Ok(reply) => replies.push(reply),
                Err(err) => {
                    warn!("Failed to process message: {}", err);
                    replies.push(Reply::Status(StatusReply {
                        status: Status {
                            code: 500,
                            detail: Some(err.to_string()),
                        },
                    }));
                }
            }
        }

        Response {
            status: Some(Status::ok()),
            replies,
        }
    }

    pub async fn process_message(&self, message: RawMessage) -> Result<Reply, HandleMessageError> {
        let message = message.validate().await?;

        match message.read().descriptor {
            Descriptor::RecordsDelete(_) => match message {
                ValidatedMessage::AttestedAuthorized(message) => {
                    handle_records_delete(&self.data_store, &self.message_store, message).await
                }
                ValidatedMessage::Authorized(message) => {
                    handle_records_delete(&self.data_store, &self.message_store, message).await
                }
                _ => Err(HandleMessageError::Unauthorized),
            },
            Descriptor::RecordsRead(_) => {
                handle_records_read(&self.data_store, &self.message_store, message).await
            }
            Descriptor::RecordsQuery(_) => handle_records_query(&self.message_store, message).await,
            Descriptor::RecordsWrite(_) => match message {
                ValidatedMessage::AttestedAuthorized(message) => {
                    handle_records_write(&self.data_store, &self.message_store, message).await
                }
                ValidatedMessage::Authorized(message) => {
                    handle_records_write(&self.data_store, &self.message_store, message).await
                }
                _ => Err(HandleMessageError::Unauthorized),
            },
            _ => Err(HandleMessageError::UnsupportedInterface),
        }
    }
}
