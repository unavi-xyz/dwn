//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! ## Usage
//!
//! ```
//! use dwn::{
//!     handlers::Status,
//!     message::{Data, MessageBuilder, descriptor::RecordsWrite},
//!     store::SurrealDB,
//!     util::DidKey,
//!     DWN
//! };
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a DWN, using an embedded SurrealDB instance as both the data and message store.
//!     let db = SurrealDB::new().await.unwrap();
//!     let dwn = DWN {
//!         data_store: db.clone(),
//!         message_store: db,
//!     };
//!
//!     // Generate a DID.
//!     let did_key = DidKey::new().unwrap();
//!
//!     // Store a record in the DWN.
//!     let message = MessageBuilder::new::<RecordsWrite>()
//!         .authorize(did_key.kid.clone(), &did_key.jwk)
//!         .data(Data::Base64("Hello, world!".to_string()))
//!         .build()
//!         .unwrap();
//!
//!     let reply = dwn
//!         .process_message(&did_key.did, message)
//!         .await
//!         .unwrap();
//!
//!     assert_eq!(reply.status().code, 200);
//! }
//! ```

use handlers::{
    records::{
        commit::RecordsCommitHandler, delete::RecordsDeleteHandler, query::RecordsQueryHandler,
        read::RecordsReadHandler, write::RecordsWriteHandler,
    },
    HandlerError, MethodHandler, Reply,
};
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
    ) -> Result<Reply, HandleMessageError> {
        match &message.descriptor {
            Descriptor::RecordsCommit(_) => {
                let handler = RecordsCommitHandler {
                    data_store: &self.data_store,
                    message_store: &self.message_store,
                };
                let reply = handler.handle(tenant, message).await?;
                Ok(reply.into())
            }
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

#[cfg(test)]
mod tests {
    use crate::{store::SurrealDB, DWN};

    pub async fn create_dwn() -> DWN<SurrealDB, SurrealDB> {
        let db = SurrealDB::new().await.expect("Failed to create SurrealDB");
        DWN {
            data_store: db.clone(),
            message_store: db,
        }
    }
}
