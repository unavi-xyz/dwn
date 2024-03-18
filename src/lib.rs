//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! ## Usage
//!
//! ```
//! use std::sync::Arc;
//!
//! use dwn::{store::SurrealDB, Actor, DWN};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a DWN, using an embedded SurrealDB for both the data and message store.
//!     let db = SurrealDB::new().await.unwrap();
//!     let dwn = DWN {
//!         data_store: db.clone(),
//!         message_store: db,
//!     };
//!
//!     // Create an actor to send messages.
//!     // Here we generate a new `did:key` for the actor's identity,
//!     // but you could use any DID method.
//!     let actor = Actor::new_did_key(dwn).unwrap();
//!
//!     // Write a new record.
//!     let data = "Hello, world!".bytes().collect::<Vec<_>>();
//!
//!     let write = actor
//!         .write()
//!         .data(data.clone())
//!         .send()
//!         .await
//!         .unwrap();
//!
//!     assert_eq!(write.reply.status.code, 200);
//!
//!     // Read the record.
//!     let read = actor.read(write.entry_id).await.unwrap();
//!
//!     assert_eq!(read.status.code, 200);
//!     assert_eq!(read.data, Some(data));
//! }
//! ```

use handlers::{
    records::{
        delete::RecordsDeleteHandler, query::RecordsQueryHandler, read::RecordsReadHandler,
        write::RecordsWriteHandler,
    },
    HandlerError, MethodHandler, Reply,
};
use message::{descriptor::Descriptor, Message};
use store::{DataStore, MessageStore};
use thiserror::Error;

mod actor;
pub mod handlers;
pub mod message;
pub mod store;
pub mod util;

pub use actor::{Actor, MessageSendError};

#[derive(Clone)]
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
    ) -> Result<Reply, HandleMessageError> {
        match &message.descriptor {
            Descriptor::RecordsDelete(_) => {
                let handler = RecordsDeleteHandler {
                    data_store: &self.data_store,
                    message_store: &self.message_store,
                };
                let reply = handler.handle(tenant, message).await?;
                Ok(reply.into())
            }
            Descriptor::RecordsQuery(_) => {
                let handler = RecordsQueryHandler {
                    data_store: &self.data_store,
                    message_store: &self.message_store,
                };
                let reply = handler.handle(tenant, message).await?;
                Ok(reply.into())
            }
            Descriptor::RecordsRead(_) => {
                let handler = RecordsReadHandler {
                    data_store: &self.data_store,
                    message_store: &self.message_store,
                };
                let reply = handler.handle(tenant, message).await?;
                Ok(reply.into())
            }
            Descriptor::RecordsWrite(_) => {
                let handler = RecordsWriteHandler {
                    data_store: &self.data_store,
                    message_store: &self.message_store,
                };
                let reply = handler.handle(tenant, message).await?;
                Ok(reply.into())
            }
            _ => Err(HandleMessageError::UnsupportedInterface),
        }
    }
}
