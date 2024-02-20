use didkit::JWK;
use thiserror::Error;
use time::OffsetDateTime;

use crate::util::EncodeError;

use super::{
    descriptor::{Descriptor, RecordsCommit},
    AuthError, Data, Message,
};

pub struct MessageBuilder<'a> {
    authorizer: Option<(String, &'a JWK)>,
    data: Option<Data>,
    descriptor: Descriptor,
    record_id: Option<String>,
}

#[derive(Debug, Error)]
pub enum MessageBuildError {
    #[error("Cbord encode error: {0}")]
    Encode(#[from] EncodeError),
    #[error("Auth error: {0}")]
    Auth(#[from] AuthError),
}

impl<'a> MessageBuilder<'a> {
    pub fn new(descriptor: impl Into<Descriptor>) -> Self {
        MessageBuilder {
            authorizer: None,
            data: None,
            descriptor: descriptor.into(),
            record_id: None,
        }
    }

    /// Create a new RecordsCommit message for a given parent message.
    pub fn new_commit(parent: &Message) -> Result<Self, EncodeError> {
        let entry_id = parent.generate_record_id()?;

        Ok(MessageBuilder {
            authorizer: None,
            data: None,
            descriptor: RecordsCommit::new(entry_id).into(),
            record_id: Some(parent.record_id.clone()),
        })
    }

    pub fn authorize(mut self, kid: String, jwk: &'a JWK) -> Self {
        self.authorizer = Some((kid, jwk));
        self
    }

    pub fn data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the record ID for the message.
    /// If None, the record ID will be automatically generated from the message.
    pub fn record_id(mut self, record_id: Option<String>) -> Self {
        self.record_id = record_id;
        self
    }

    pub fn build(self) -> Result<Message, MessageBuildError> {
        let mut msg = Message {
            attestation: None,
            authorization: None,
            data: self.data,
            descriptor: self.descriptor,
            record_id: self.record_id.unwrap_or_default(),
        };

        let timestamp = OffsetDateTime::now_utc();

        match &mut msg.descriptor {
            Descriptor::RecordsWrite(desc) => {
                desc.message_timestamp = timestamp;

                if let Some(data) = &msg.data {
                    let cid = data.cid()?;
                    desc.data_cid = Some(cid.to_string());
                }
            }
            Descriptor::RecordsCommit(desc) => {
                desc.message_timestamp = timestamp;
            }
            Descriptor::RecordsDelete(desc) => {
                desc.message_timestamp = timestamp;
            }
            Descriptor::RecordsRead(desc) => {
                desc.message_timestamp = timestamp;
            }
            Descriptor::RecordsQuery(desc) => {
                desc.message_timestamp = timestamp;
            }
            _ => {}
        }

        if msg.record_id.is_empty() {
            msg.record_id = msg.generate_record_id()?;
        }

        if let Some((kid, jwk)) = self.authorizer {
            msg.authorize(kid, jwk)?;
        }

        Ok(msg)
    }
}
