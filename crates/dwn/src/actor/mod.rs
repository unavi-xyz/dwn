use std::sync::Arc;

use didkit::JWK;
use openssl::error::ErrorStack;
use thiserror::Error;

use crate::{
    handlers::{RecordsQueryReply, RecordsReadReply, Reply, StatusReply},
    message::{
        descriptor::{Filter, RecordsDelete, RecordsQuery, RecordsRead},
        AuthError, Message, SignError,
    },
    store::{DataStore, MessageStore},
    util::EncodeError,
    HandleMessageError, DWN,
};

mod create;
mod did_key;

pub use create::{CreateRecord, Encryption};

use self::create::build_write;

/// Identity actor.
/// Holds a DID and associated keys.
/// Provides methods for interacting with a DWN using the actor's DID.
pub struct Actor<D: DataStore, M: MessageStore> {
    pub attestation: VerifiableCredential,
    pub authorization: VerifiableCredential,
    pub did: String,
    pub dwn: Arc<DWN<D, M>>,
}

pub struct VerifiableCredential {
    pub jwk: JWK,
    pub kid: String,
}

impl<D: DataStore, M: MessageStore> Actor<D, M> {
    /// Generates a new `did:key` actor.
    pub fn new_did_key(dwn: Arc<DWN<D, M>>) -> Result<Actor<D, M>, did_key::DidKeygenError> {
        let did_key = did_key::DidKey::new()?;
        Ok(Actor {
            attestation: VerifiableCredential {
                jwk: did_key.jwk.clone(),
                kid: did_key.kid.clone(),
            },
            authorization: VerifiableCredential {
                jwk: did_key.jwk,
                kid: did_key.kid,
            },
            did: did_key.did,
            dwn,
        })
    }

    pub async fn create(&self, create: CreateRecord<'_>) -> Result<CreateResult, MessageSendError> {
        let msg = create.build(self)?;
        let record_id = msg.record_id.clone();

        let reply = self.dwn.process_message(msg).await?;

        match reply {
            Reply::Status(reply) => Ok(CreateResult { record_id, reply }),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }

    pub async fn delete(&self, record_id: String) -> Result<StatusReply, MessageSendError> {
        let mut msg = Message::new(RecordsDelete::new(record_id));
        msg.record_id = msg.entry_id()?;

        msg.authorize(self.authorization.kid.clone(), &self.authorization.jwk)?;

        let reply = self.dwn.process_message(msg).await?;

        match reply {
            Reply::Status(reply) => Ok(reply),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }

    pub async fn query(&self, filter: Filter) -> Result<RecordsQueryReply, MessageSendError> {
        let mut msg = Message::new(RecordsQuery::new(filter));

        if msg.record_id.is_empty() {
            msg.record_id = msg.entry_id()?;
        }

        msg.authorize(self.authorization.kid.clone(), &self.authorization.jwk)?;

        let reply = self.dwn.process_message(msg).await?;

        match reply {
            Reply::RecordsQuery(reply) => Ok(reply),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }

    pub async fn read(&self, record_id: String) -> Result<Box<RecordsReadReply>, MessageSendError> {
        let mut msg = Message::new(RecordsRead::new(record_id));
        msg.record_id = msg.entry_id()?;

        msg.authorize(self.authorization.kid.clone(), &self.authorization.jwk)?;

        let reply = self.dwn.process_message(msg).await?;

        match reply {
            Reply::RecordsRead(reply) => Ok(reply),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }

    pub async fn update(
        &self,
        record_id: String,
        entry_id: String,
        create: CreateRecord<'_>,
    ) -> Result<UpdateResult, MessageSendError> {
        let msg = build_write(self, create, Some(entry_id), Some(record_id))?;
        let entry_id = msg.entry_id()?;

        let reply = self.dwn.process_message(msg).await?;

        match reply {
            Reply::Status(reply) => Ok(UpdateResult { entry_id, reply }),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
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
pub enum MessageSendError {
    #[error(transparent)]
    MessageSign(#[from] SignError),
    #[error(transparent)]
    MessageAuth(#[from] AuthError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    Handler(#[from] HandleMessageError),
    #[error("Invalid reply: {:?}", 0)]
    InvalidReply(Reply),
    #[error(transparent)]
    OpenSSL(#[from] ErrorStack),
}
