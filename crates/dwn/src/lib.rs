//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! ## Usage
//!
//! ```
//! use dwn::{actor::{Actor, CreateRecord}, store::SurrealStore, DWN};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a DWN, using an in-memory SurrealDB instance for storage.
//!     let store = SurrealStore::new().await.unwrap();
//!     let dwn = DWN::from(store);
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
use message::{descriptor::Descriptor, Message, Request, ValidateError};
use reqwest::Client;
use store::{DataStore, DataStoreError, MessageStore, MessageStoreError};
use sync::RemoteSync;
use thiserror::Error;
use tokio::sync::mpsc::{error::SendError, Sender};

pub mod actor;
pub mod handlers;
pub mod message;
pub mod store;
pub mod sync;
pub mod util;

use tracing::warn;
use util::EncodeError;

use crate::handlers::StatusReply;

#[derive(Clone)]
pub struct DWN<D: DataStore, M: MessageStore> {
    pub data_store: D,
    pub message_store: M,
    remote: Option<Remote>,
}

#[derive(Clone)]
struct Remote {
    client: Client,
    sender: Sender<Message>,
    url: String,
}

impl Remote {
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
            remote: None,
        }
    }
}

impl<D: DataStore, M: MessageStore> DWN<D, M> {
    pub fn new(data_store: D, message_store: M) -> Self {
        Self {
            data_store,
            message_store,
            remote: None,
        }
    }

    pub fn sync_with(&mut self, remote_url: String) -> RemoteSync {
        let remote_sync = RemoteSync::new(remote_url.clone());

        self.remote = Some(Remote {
            client: Client::new(),
            sender: remote_sync.sender.clone(),
            url: remote_url,
        });

        remote_sync
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
                if let Some(remote) = &self.remote {
                    remote
                        .sender
                        .send(message.clone())
                        .await
                        .map_err(Box::new)?;
                }

                handle_records_delete(&self.data_store, &self.message_store, message).await
            }
            Descriptor::RecordsRead(_) => {
                match handle_records_read(&self.data_store, &self.message_store, message.clone())
                    .await
                {
                    Ok(reply) => Ok(reply),
                    Err(err) => {
                        // If read fails, try remote.
                        if let Some(remote) = &self.remote {
                            let response = remote.send(vec![message]).await?;
                            Ok(response.replies.into_iter().next().unwrap())
                        } else {
                            Err(err)
                        }
                    }
                }
            }
            Descriptor::RecordsQuery(_) => handle_records_query(&self.message_store, message).await,
            Descriptor::RecordsWrite(_) => {
                if let Some(remote) = &self.remote {
                    remote
                        .sender
                        .send(message.clone())
                        .await
                        .map_err(Box::new)?;
                }

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
