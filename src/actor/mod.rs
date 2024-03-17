use didkit::JWK;
use thiserror::Error;

use crate::{
    handlers::{RecordsReadReply, Reply, StatusReply},
    message::{
        descriptor::{RecordsCommit, RecordsRead},
        AuthError, Message,
    },
    store::{DataStore, MessageStore},
    util::EncodeError,
    HandleMessageError, DWN,
};

use self::{query::RecordsQueryBuilder, write::RecordsWriteBuilder};

mod did_key;
mod query;
mod write;

pub struct Actor<D: DataStore, M: MessageStore> {
    pub did: String,
    pub dwn: DWN<D, M>,
    pub jwk: JWK,
    // VC DID URL for the JWK.
    pub kid: String,
}

impl<D: DataStore, M: MessageStore> Actor<D, M> {
    pub fn new_did_key(dwn: DWN<D, M>) -> Result<Actor<D, M>, did_key::DidKeygenError> {
        let did_key = did_key::DidKey::new()?;
        Ok(Actor {
            did: did_key.did,
            dwn,
            jwk: did_key.jwk,
            kid: did_key.kid,
        })
    }

    pub async fn commit(&self, parent_id: String) -> Result<StatusReply, MessageSendError> {
        let mut msg = Message::new(RecordsCommit::new(parent_id));
        msg.record_id = msg.generate_record_id()?;

        let reply = self.dwn.process_message(&self.did, msg).await?;

        match reply {
            Reply::Status(reply) => Ok(reply),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }

    pub async fn read(&self, record_id: String) -> Result<Box<RecordsReadReply>, MessageSendError> {
        let mut msg = Message::new(RecordsRead::new(record_id));
        msg.record_id = msg.generate_record_id()?;

        let reply = self.dwn.process_message(&self.did, msg).await?;

        match reply {
            Reply::RecordsRead(reply) => Ok(reply),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }

    pub fn query(&self) -> RecordsQueryBuilder<D, M> {
        RecordsQueryBuilder::new(self)
    }

    pub fn write(&self) -> RecordsWriteBuilder<D, M> {
        RecordsWriteBuilder::new(self)
    }
}

#[derive(Debug, Error)]
pub enum MessageSendError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    Handler(#[from] HandleMessageError),
    #[error("Invalid reply: {:?}", 0)]
    InvalidReply(Reply),
}
