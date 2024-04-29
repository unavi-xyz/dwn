//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! ## Usage
//!
//! ```
//! use std::sync::Arc;
//!
//! use dwn::{actor::Actor, message::Data, store::SurrealStore, DWN};
//! use surrealdb::{Surreal, engine::local::Mem};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a DWN, using in-memory SurrealDB for storage.
//!     let db = Surreal::new::<Mem>(()).await.unwrap();
//!     let store = SurrealStore::new(db).await.unwrap();
//!     let dwn = Arc::new(DWN::from(store));
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
//!         .create_record()
//!         .data(data.clone())
//!         .process()
//!         .await
//!         .unwrap();
//!
//!     assert_eq!(create.reply.status.code, 200);
//!
//!     // Read the record.
//!     let read = actor
//!         .read_record(create.record_id)
//!         .process()
//!         .await
//!         .unwrap();
//!
//!     assert_eq!(read.status.code, 200);
//!     assert_eq!(read.record.data, Some(Data::new_base64(&data)));
//! }
//! ```

use handlers::{
    protocols::{handle_protocols_configure, handle_protocols_query},
    records::{
        handle_records_delete, handle_records_query, handle_records_read, handle_records_write,
        HandleWriteOptions,
    },
};
use message::{descriptor::Descriptor, DwnRequest, Message, ValidateError};
use reply::MessageReply;
use reqwest::Client;
use store::{DataStore, DataStoreError, MessageStore, MessageStoreError};
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

pub mod actor;
mod encode;
mod handlers;
pub mod message;
pub mod reply;
pub mod store;

pub use encode::EncodeError;

pub struct DWN<D: DataStore, M: MessageStore> {
    pub client: Client,
    pub data_store: D,
    pub message_store: M,
}

impl<T: Clone + DataStore + MessageStore> From<T> for DWN<T, T> {
    fn from(store: T) -> Self {
        Self {
            client: Client::new(),
            data_store: store.clone(),
            message_store: store,
        }
    }
}

impl<D: DataStore, M: MessageStore> DWN<D, M> {
    pub fn new(data_store: D, message_store: M) -> Self {
        Self {
            client: Client::new(),
            data_store,
            message_store,
        }
    }

    pub async fn process_message(
        &self,
        request: DwnRequest,
    ) -> Result<MessageReply, HandleMessageError> {
        match &request.message.descriptor {
            Descriptor::ProtocolsConfigure(_) => {
                handle_protocols_configure(&self.data_store, &self.message_store, request).await
            }
            Descriptor::ProtocolsQuery(_) => {
                handle_protocols_query(&self.message_store, request).await
            }

            Descriptor::RecordsDelete(_) => {
                handle_records_delete(&self.data_store, &self.message_store, request).await
            }
            Descriptor::RecordsRead(_) => {
                handle_records_read(&self.data_store, &self.message_store, request).await
            }
            Descriptor::RecordsQuery(_) => handle_records_query(&self.message_store, request).await,
            Descriptor::RecordsWrite(_) => {
                handle_records_write(
                    &self.client,
                    &self.data_store,
                    &self.message_store,
                    request,
                    HandleWriteOptions::default(),
                )
                .await
            }
            _ => Err(HandleMessageError::UnsupportedInterface),
        }
    }
}

#[derive(Debug, Error)]
pub enum HandleMessageError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Unsupported interface")]
    UnsupportedInterface,
    #[error(transparent)]
    ValidateError(#[from] ValidateError),
    #[error("Invalid descriptor: {0}")]
    InvalidDescriptor(String),
    #[error("Failed to validate schema: {0}")]
    SchemaValidation(String),
    #[error(transparent)]
    DataStoreError(#[from] DataStoreError),
    #[error(transparent)]
    MessageStoreError(#[from] MessageStoreError),
    #[error(transparent)]
    CborEncode(#[from] EncodeError),
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SendError(#[from] Box<SendError<Message>>),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}
