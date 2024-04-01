use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use iana_media_types::MediaType;
use openssl::{error::ErrorStack, rand::rand_bytes};
use semver::Version;
use time::OffsetDateTime;

use crate::{
    actor::{Actor, MessageBuilder, PrepareError, ProcessMessageError},
    handlers::{MessageReply, StatusReply},
    message::{descriptor::records::RecordsWrite, Data, EncryptedData, Message},
    store::{DataStore, MessageStore},
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
    data_format: Option<MediaType>,
    encryption: Option<&'a Encryption>,
    parent_context_id: Option<String>,
    parent_id: Option<String>,
    protocol: Option<String>,
    protocol_path: Option<String>,
    protocol_version: Option<Version>,
    published: Option<bool>,
    record_id: Option<String>,
    schema: Option<String>,
    signed: bool,
    target: Option<String>,

    final_context_id: Option<String>,
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

    fn post_build(&mut self, message: &mut Message) -> Result<(), PrepareError> {
        self.final_context_id = message.context_id.clone();
        self.final_entry_id = message.entry_id()?;
        self.final_record_id = message.record_id.clone();
        Ok(())
    }

    fn create_message(&mut self) -> Result<Message, PrepareError> {
        let mut descriptor = RecordsWrite::default();
        descriptor.data_format = self.data_format.take();
        descriptor.message_timestamp = OffsetDateTime::now_utc();
        descriptor.parent_id = self.parent_id.take();
        descriptor.protocol = self.protocol.take();
        descriptor.protocol_path = self.protocol_path.take();
        descriptor.protocol_version = self.protocol_version.take();
        descriptor.published = self.published;
        descriptor.schema = self.schema.take();

        let has_protocol = descriptor.protocol.is_some()
            || descriptor.protocol_path.is_some()
            || descriptor.protocol_version.is_some();

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
            context_id: None,
            data,
            descriptor: descriptor.into(),
            record_id: self.record_id.take().unwrap_or_default(),
        };

        if msg.record_id.is_empty() {
            msg.record_id = msg.entry_id()?;
        }

        if has_protocol {
            let parent_context_id = self
                .parent_context_id
                .take()
                .map(|id| format!("{}/", id))
                .unwrap_or_default();

            msg.context_id = Some(format!("{}{}", parent_context_id, msg.record_id));
        }

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
            data_format: None,
            encryption: None,
            parent_context_id: None,
            parent_id: None,
            protocol: None,
            protocol_path: None,
            protocol_version: None,
            published: None,
            record_id: None,
            schema: None,
            signed: true,
            target: None,

            final_context_id: None,
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

    /// Format of the data.
    pub fn data_format(mut self, data_format: MediaType) -> Self {
        self.data_format = Some(data_format);
        self
    }

    /// Encryption to use on the data.
    pub fn encryption(mut self, encryption: &'a Encryption) -> Self {
        self.encryption = Some(encryption);
        self
    }

    pub fn parent_context_id(mut self, parent_context_id: String) -> Self {
        self.parent_context_id = Some(parent_context_id);
        self
    }

    pub fn protocol(mut self, protocol: String, version: Version, path: String) -> Self {
        self.protocol = Some(protocol);
        self.protocol_version = Some(version);
        self.protocol_path = Some(path);
        self
    }

    /// Whether the message should be published.
    /// This makes the message publicly readable.
    /// Defaults to false.
    pub fn published(mut self, published: bool) -> Self {
        self.published = Some(published);
        self
    }

    /// URI of a JSON schema for the data.
    /// All future updates to this record must conform to this schema.
    /// Can only be set once in the first write.
    pub fn schema(mut self, schema: String) -> Self {
        self.schema = Some(schema);
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
            MessageReply::Status(reply) => reply,
            _ => return Err(ProcessMessageError::InvalidReply),
        };

        Ok(WriteResponse {
            context_id: self.final_context_id,
            entry_id: self.final_entry_id,
            record_id: self.final_record_id,
            reply,
        })
    }
}

#[derive(Debug)]
pub struct WriteResponse {
    pub context_id: Option<String>,
    pub entry_id: String,
    pub record_id: String,
    pub reply: StatusReply,
}
