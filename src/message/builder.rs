use didkit::JWK;
use thiserror::Error;
use time::OffsetDateTime;

use crate::util::CborEncodeError;

use super::{descriptor::Descriptor, AuthError, Data, Message};

pub struct MessageBuilder<'a> {
    data: Option<Data>,
    descriptor: Descriptor,
    authorizer: Option<(String, &'a JWK)>,
}

#[derive(Debug, Error)]
pub enum MessageBuildError {
    #[error("Cbord encode error: {0}")]
    Encode(#[from] CborEncodeError),
    #[error("Auth error: {0}")]
    Auth(#[from] AuthError),
}

impl<'a> MessageBuilder<'a> {
    pub fn new(descriptor: impl Into<Descriptor>) -> Self {
        MessageBuilder {
            authorizer: None,
            data: None,
            descriptor: descriptor.into(),
        }
    }

    pub fn data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }

    pub fn authorize(mut self, kid: String, jwk: &'a JWK) -> Self {
        self.authorizer = Some((kid, jwk));
        self
    }

    pub fn build(self) -> Result<Message, MessageBuildError> {
        let mut msg = Message {
            data: self.data,
            descriptor: self.descriptor,
            attestation: None,
            authorization: None,
            record_id: String::new(),
        };

        let timestamp = OffsetDateTime::now_utc();

        match &mut msg.descriptor {
            Descriptor::RecordsWrite(desc) => {
                desc.message_timestamp = timestamp;

                if let Some(data) = &msg.data {
                    desc.data_cid = Some(data.cid()?.to_string());
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
        }

        msg.record_id = msg.generate_record_id()?;

        if let Some((kid, jwk)) = self.authorizer {
            msg.authorize(kid, jwk)?;
        }

        Ok(msg)
    }
}
