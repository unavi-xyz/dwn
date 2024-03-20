use didkit::JWK;
use openssl::error::ErrorStack;
use thiserror::Error;

use crate::{
    handlers::{RecordsReadReply, Reply, StatusReply},
    message::{
        descriptor::{RecordsDelete, RecordsRead},
        AuthError, Message, SignError,
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
    pub attestation: VerifiableCredential,
    pub authorization: VerifiableCredential,
    pub did: String,
    pub dwn: DWN<D, M>,
}

pub struct VerifiableCredential {
    pub jwk: JWK,
    pub kid: String,
}

impl<D: DataStore, M: MessageStore> Actor<D, M> {
    pub fn new_did_key(dwn: DWN<D, M>) -> Result<Actor<D, M>, did_key::DidKeygenError> {
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

    pub async fn delete(&self, record_id: String) -> Result<StatusReply, MessageSendError> {
        let mut msg = Message::new(RecordsDelete::new(record_id));
        msg.record_id = msg.generate_record_id()?;

        msg.sign(self.attestation.kid.clone(), &self.attestation.jwk)?;
        msg.authorize(self.authorization.kid.clone(), &self.authorization.jwk)?;

        let reply = self.dwn.process_message(msg).await?;

        match reply {
            Reply::Status(reply) => Ok(reply),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }

    pub async fn read(&self, record_id: String) -> Result<Box<RecordsReadReply>, MessageSendError> {
        let mut msg = Message::new(RecordsRead::new(record_id));
        msg.record_id = msg.generate_record_id()?;

        msg.sign(self.attestation.kid.clone(), &self.attestation.jwk)?;
        msg.authorize(self.authorization.kid.clone(), &self.authorization.jwk)?;

        let reply = self.dwn.process_message(msg).await?;

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
