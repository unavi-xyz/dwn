use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use openssl::{error::ErrorStack, rand::rand_bytes};
use time::OffsetDateTime;

use crate::{
    handlers::{Reply, StatusReply},
    message::{
        data::{Data, EncryptedData},
        descriptor::RecordsWrite,
        Message,
    },
    store::{DataStore, MessageStore},
};

use super::{
    builder::{MessageBuilder, ProcessMessageError},
    Actor, PrepareError,
};

#[derive(Clone)]
pub enum Encryption {
    Aes256Gcm(Vec<u8>),
}

impl Encryption {
    pub fn generate_aes256() -> Result<Self, ErrorStack> {
        let mut key = vec![0; 32]; // AES-256 key size
        rand_bytes(&mut key)?;
        Ok(Self::Aes256Gcm(key))
    }
}

pub struct RecordsWriteBuilder<'a, D: DataStore, M: MessageStore> {
    actor: &'a Actor<D, M>,
    authorized: bool,
    data: Option<Vec<u8>>,
    encryption: Option<&'a Encryption>,
    parent_id: Option<String>,
    published: bool,
    record_id: Option<String>,
    signed: bool,
    target: Option<String>,

    final_entry_id: String,
    final_record_id: String,
}

impl<'a, D: DataStore, M: MessageStore> MessageBuilder for RecordsWriteBuilder<'a, D, M> {
    fn get_actor(&self) -> &Actor<impl DataStore, impl MessageStore> {
        self.actor
    }

    fn get_authorized(&self) -> bool {
        self.authorized
    }
    fn authorized(mut self, authorized: bool) -> Self {
        self.authorized = authorized;
        self
    }

    fn get_target(&self) -> Option<String> {
        self.target.clone()
    }
    fn target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    fn message_hook(&mut self, message: &mut Message) -> Result<(), PrepareError> {
        self.final_entry_id = message.entry_id()?;
        self.final_record_id = message.record_id.clone();
        Ok(())
    }

    fn build(&mut self) -> Result<Message, PrepareError> {
        let mut descriptor = RecordsWrite::default();
        descriptor.message_timestamp = OffsetDateTime::now_utc();
        descriptor.parent_id = self.parent_id.take();
        descriptor.published = Some(self.published);

        let mut data = None;

        if let Some(bytes) = self.data.take() {
            match self.encryption {
                Some(Encryption::Aes256Gcm(key)) => {
                    let encrypted = EncryptedData::encrypt_aes(&bytes, key)?;
                    data = Some(Data::Encrypted(encrypted));
                }
                None => {
                    let encoded = URL_SAFE_NO_PAD.encode(bytes);
                    data = Some(Data::Base64(encoded));
                }
            }
        }

        if let Some(data) = &data {
            let cid = data.cid()?;
            descriptor.data_cid = Some(cid.to_string());
        }

        let mut msg = Message {
            attestation: None,
            authorization: None,
            data,
            descriptor: descriptor.into(),
            record_id: self.record_id.take().unwrap_or_default(),
        };

        if self.signed {
            msg.sign(
                self.actor.attestation.key_id.clone(),
                &self.actor.attestation.jwk,
            )?;
        }

        Ok(msg)
    }
}

impl<'a, D: DataStore, M: MessageStore> RecordsWriteBuilder<'a, D, M> {
    pub fn new(actor: &'a Actor<D, M>) -> Self {
        RecordsWriteBuilder {
            actor,
            authorized: true,
            data: None,
            encryption: None,
            parent_id: None,
            published: false,
            record_id: None,
            signed: true,
            target: None,

            final_entry_id: String::new(),
            final_record_id: String::new(),
        }
    }

    pub fn new_update(actor: &'a Actor<D, M>, record_id: String, parent_id: String) -> Self {
        let mut builder = RecordsWriteBuilder::new(actor);
        builder.record_id = Some(record_id);
        builder.parent_id = Some(parent_id);
        builder
    }

    /// Data to be written.
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Encryption to use on the data.
    pub fn encryption(mut self, encryption: &'a Encryption) -> Self {
        self.encryption = Some(encryption);
        self
    }

    /// Whether the message should be published.
    /// This makes the message publicly readable.
    /// Defaults to false.
    pub fn published(mut self, published: bool) -> Self {
        self.published = published;
        self
    }

    /// Whether the message should be signed.
    /// Defaults to true.
    pub fn signed(mut self, signed: bool) -> Self {
        self.signed = signed;
        self
    }

    pub async fn process(mut self) -> Result<WriteResponse, ProcessMessageError> {
        let reply = MessageBuilder::process(&mut self).await?;

        let reply = match reply {
            Reply::Status(reply) => reply,
            _ => return Err(ProcessMessageError::InvalidReply),
        };

        Ok(WriteResponse {
            entry_id: self.final_entry_id,
            record_id: self.final_record_id,
            reply,
        })
    }
}

pub struct WriteResponse {
    pub entry_id: String,
    pub record_id: String,
    pub reply: StatusReply,
}
