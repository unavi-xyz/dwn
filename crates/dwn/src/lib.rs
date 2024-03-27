//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! ## Usage
//!
//! ```
//! use std::sync::Arc;
//!
//! use dwn::{actor::{Actor, CreateRecord}, message::data::Data, store::SurrealStore, DWN};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a DWN, using an in-memory SurrealDB instance for storage.
//!     let store = SurrealStore::new().await.unwrap();
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
//!     assert_eq!(read.record.data, Some(Data::new_base64(&data)));
//! }
//! ```

use handlers::{
    records::{
        delete::handle_records_delete, query::handle_records_query, read::handle_records_read,
        write::handle_records_write,
    },
    Reply, Response, Status,
};
use message::{descriptor::Descriptor, Message, Request, ValidateError};
use reqwest::Client;
use store::{DataStore, DataStoreError, MessageStore, MessageStoreError};
use sync::RemoteSync;
use thiserror::Error;
use tokio::sync::{
    mpsc::{error::SendError, Sender},
    RwLock,
};

pub mod actor;
pub mod handlers;
pub mod message;
pub mod store;
pub mod sync;
pub mod util;

use tracing::warn;
use util::EncodeError;

use crate::handlers::StatusReply;

pub struct DWN<D: DataStore, M: MessageStore> {
    pub data_store: D,
    pub message_store: M,
    pub remote: RwLock<Option<Remote>>,
}

pub struct Remote {
    pub client: Client,
    pub sender: Sender<Message>,
    pub url: String,
}

impl Remote {
    pub fn from_remote_sync(remote_sync: &RemoteSync) -> Self {
        Self {
            client: Client::new(),
            sender: remote_sync.sender.clone(),
            url: remote_sync.remote_url.clone(),
        }
    }

    async fn send(&self, messages: Vec<Message>) -> Result<Response, reqwest::Error> {
        self.client
            .post(&self.url)
            .json(&Request { messages })
            .send()
            .await?
            .json::<Response>()
            .await
    }
}

impl<T: Clone + DataStore + MessageStore> From<T> for DWN<T, T> {
    fn from(store: T) -> Self {
        Self {
            data_store: store.clone(),
            message_store: store,
            remote: RwLock::new(None),
        }
    }
}

impl<D: DataStore, M: MessageStore> DWN<D, M> {
    pub fn new(data_store: D, message_store: M) -> Self {
        Self {
            data_store,
            message_store,
            remote: RwLock::new(None),
        }
    }

    pub async fn set_remote(&self, remote_url: String) -> RemoteSync {
        let remote_sync = RemoteSync::new(remote_url.clone());
        let remote = Remote::from_remote_sync(&remote_sync);

        *self.remote.write().await = Some(remote);

        remote_sync
    }

    /// Sends a message to the remote node, if one is set.
    /// Clones the message if it is sent.
    pub async fn send_to_remote(&self, message: &Message) -> Result<(), HandleMessageError> {
        if let Some(remote) = &self.remote.read().await.as_ref() {
            remote
                .sender
                .send(message.clone())
                .await
                .map_err(Box::new)?;
        }

        Ok(())
    }

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

    pub async fn process_message(&self, message: Message) -> Result<Reply, HandleMessageError> {
        message.validate().await?;

        match &message.descriptor {
            Descriptor::RecordsDelete(_) => {
                self.send_to_remote(&message).await?;
                handle_records_delete(&self.data_store, &self.message_store, message).await
            }
            Descriptor::RecordsRead(_) => {
                match handle_records_read(&self.data_store, &self.message_store, message.clone())
                    .await
                {
                    Ok(reply) => Ok(reply),
                    Err(err) => {
                        // If read fails, check remote.
                        if let Some(remote) = self.remote.read().await.as_ref() {
                            let tenant = message.tenant();
                            let response = remote.send(vec![message]).await?;

                            match response.replies.into_iter().next() {
                                Some(Reply::RecordsRead(reply)) => {
                                    // Add record to local store.
                                    if let Some(tenant) = tenant {
                                        // TODO: Only store data by default if under a certain size.
                                        // TODO: Add a flag to enable or disable storing data.

                                        self.message_store
                                            .put(tenant, reply.record.clone(), &self.data_store)
                                            .await?;
                                    }

                                    Ok(Reply::RecordsRead(reply))
                                }
                                Some(_) => Err(HandleMessageError::InvalidDescriptor(
                                    "Unexpected reply from remote".to_string(),
                                )),
                                None => Err(HandleMessageError::InvalidDescriptor(
                                    "No reply from remote".to_string(),
                                )),
                            }
                        } else {
                            Err(err)
                        }
                    }
                }
            }
            Descriptor::RecordsQuery(_) => handle_records_query(&self.message_store, message).await,
            Descriptor::RecordsWrite(_) => {
                self.send_to_remote(&message).await?;
                handle_records_write(&self.data_store, &self.message_store, message).await
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
    #[error("Invalid descriptor")]
    InvalidDescriptor(String),
    #[error(transparent)]
    DataStoreError(#[from] DataStoreError),
    #[error(transparent)]
    MessageStoreError(#[from] MessageStoreError),
    #[error(transparent)]
    CborEncode(#[from] EncodeError),
    #[error(transparent)]
    SendError(#[from] Box<SendError<Message>>),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}
