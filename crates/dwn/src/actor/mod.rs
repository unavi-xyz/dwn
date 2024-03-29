use std::{collections::HashSet, sync::Arc};

use didkit::JWK;
use openssl::error::ErrorStack;
use reqwest::Client;
use thiserror::Error;

use crate::{
    handlers::{Reply, StatusReply},
    message::{
        descriptor::{Descriptor, Filter},
        AuthError, Message, Request, SignError,
    },
    store::{DataStore, MessageStore, MessageStoreError},
    util::EncodeError,
    HandleMessageError, DWN,
};

mod builder;
mod delete;
mod did_key;
mod query;
mod read;
mod remote;
mod write;

pub use builder::MessageBuilder;

use self::{
    builder::ProcessMessageError, delete::RecordsDeleteBuilder, query::RecordsQueryBuilder,
    read::RecordsReadBuilder, remote::Remote, write::RecordsWriteBuilder,
};

pub use delete::DeleteResponse;
pub use write::{Encryption, WriteResponse};

/// Identity actor.
/// Holds a DID and associated keys.
/// Provides methods for interacting with the DID's DWN.
pub struct Actor<D: DataStore, M: MessageStore> {
    pub attestation: VerifiableCredential,
    pub authorization: VerifiableCredential,
    pub client: Client,
    pub did: String,
    pub dwn: Arc<DWN<D, M>>,
    pub remotes: Vec<Remote>,
}

pub struct VerifiableCredential {
    pub jwk: JWK,
    pub key_id: String,
}

impl<D: DataStore, M: MessageStore> Actor<D, M> {
    /// Generates a new `did:key` actor.
    pub fn new_did_key(dwn: Arc<DWN<D, M>>) -> Result<Actor<D, M>, did_key::DidKeygenError> {
        let did_key = did_key::DidKey::new()?;
        Ok(Actor {
            attestation: VerifiableCredential {
                jwk: did_key.jwk.clone(),
                key_id: did_key.key_id.clone(),
            },
            authorization: VerifiableCredential {
                jwk: did_key.jwk,
                key_id: did_key.key_id,
            },
            client: Client::new(),
            did: did_key.did,
            dwn,
            remotes: Vec::new(),
        })
    }

    pub fn add_remote(&mut self, remote_url: String) {
        let remote = Remote::new(remote_url.clone());
        self.remotes.push(remote);
    }

    pub fn remove_remote(&mut self, remote_url: &str) {
        self.remotes.retain(|remote| remote.url() != remote_url);
    }

    /// Sync the local DWN with the actor's remote DWNs.
    pub async fn sync(&self) -> Result<(), SyncError> {
        // Push to remotes.
        for remote in &self.remotes {
            while let Ok(message) = remote.receiver.write().await.try_recv() {
                self.send_message(message, remote.url()).await?;
            }
        }

        // Pull from remotes.
        let mut record_ids = HashSet::new();

        for message in self
            .dwn
            .message_store
            .query(self.did.clone(), true, Filter::default())
            .await?
        {
            record_ids.insert(message.record_id);
        }

        for remote in &self.remotes {
            let url = remote.url();

            for record_id in record_ids.iter() {
                let message = self.read(record_id.clone()).build()?;
                self.send_message(message, url).await?;
            }
        }

        Ok(())
    }

    /// Queue a message to be sent to remote DWNs.
    async fn remote_queue(&self, message: &Message) -> Result<(), HandleMessageError> {
        for remote in &self.remotes {
            remote
                .sender
                .send(message.clone())
                .await
                .map_err(Box::new)?;
        }

        Ok(())
    }

    /// Process a message in the local DWN.
    async fn process_message(&self, message: Message) -> Result<Reply, HandleMessageError> {
        match &message.descriptor {
            Descriptor::RecordsDelete(_) => {
                self.remote_queue(&message).await?;
            }
            Descriptor::RecordsWrite(_) => {
                self.remote_queue(&message).await?;
            }
            _ => {}
        }

        self.dwn
            .process_message(Request {
                target: self.did.clone(),
                message,
            })
            .await
    }

    /// Sends a message to a remote DWN.
    async fn send_message(&self, message: Message, url: &str) -> Result<Reply, reqwest::Error> {
        let request = Request {
            target: self.did.clone(),
            message,
        };

        self.client
            .post(url)
            .json(&request)
            .send()
            .await?
            .json::<Reply>()
            .await
    }

    pub fn create(&self) -> RecordsWriteBuilder<D, M> {
        RecordsWriteBuilder::new(self)
    }

    pub fn delete(&self, record_id: String) -> RecordsDeleteBuilder<D, M> {
        RecordsDeleteBuilder::new(self, record_id)
    }

    pub fn query(&self, filter: Filter) -> RecordsQueryBuilder<D, M> {
        RecordsQueryBuilder::new(self, filter)
    }

    pub fn read(&self, record_id: String) -> RecordsReadBuilder<D, M> {
        RecordsReadBuilder::new(self, record_id)
    }

    pub fn update(&self, record_id: String, parent_id: String) -> RecordsWriteBuilder<D, M> {
        RecordsWriteBuilder::new_update(self, record_id, parent_id)
    }
}

pub struct CreateResult {
    pub record_id: String,
    pub reply: StatusReply,
}

pub struct UpdateResult {
    pub entry_id: String,
    pub reply: StatusReply,
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error(transparent)]
    MessageStore(#[from] MessageStoreError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    ProcessMessage(#[from] ProcessMessageError),
}

#[derive(Debug, Error)]
pub enum PrepareError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Sign(#[from] SignError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    OpenSSL(#[from] ErrorStack),
}
