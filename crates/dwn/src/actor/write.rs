use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
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

use super::{Actor, MessageSendError};

pub struct RecordsWriteBuilder<'a, D: DataStore, M: MessageStore> {
    actor: &'a Actor<D, M>,
    data: Option<Vec<u8>>,
    encrypted: bool,
    parent_id: Option<String>,
    published: bool,
    record_id: Option<String>,
    store: bool,
}

impl<'a, D: DataStore, M: MessageStore> RecordsWriteBuilder<'a, D, M> {
    pub fn new(actor: &'a Actor<D, M>) -> Self {
        RecordsWriteBuilder {
            actor,
            data: None,
            encrypted: false,
            parent_id: None,
            published: false,
            record_id: None,
            store: true,
        }
    }

    /// Data to be written.
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Whether the data should be encrypted.
    /// Defaults to false.
    /// If set to true, the data will be encrypted using a generated key.
    pub fn encrypt(mut self, encrypt: bool) -> Self {
        self.encrypted = encrypt;
        self
    }

    /// Parent record ID.
    /// Must be set when updating a record.
    pub fn parent_id(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Whether the record should be published.
    /// This makes the record public for anyone to read.
    /// Defaults to false.
    pub fn published(mut self, published: bool) -> Self {
        self.published = published;
        self
    }

    /// Record ID to write to.
    /// If not provided, a new record ID will be generated.
    pub fn record_id(mut self, record_id: String) -> Self {
        self.record_id = Some(record_id);
        self
    }

    /// Whether to store the message locally.
    /// Defaults to true.
    pub fn store(mut self, store: bool) -> Self {
        self.store = store;
        self
    }

    /// Send the message to the DWN.
    pub async fn send(self) -> Result<WriteResult, MessageSendError> {
        let mut descriptor = RecordsWrite::default();
        descriptor.message_timestamp = OffsetDateTime::now_utc();
        descriptor.parent_id = self.parent_id.clone();
        descriptor.published = Some(self.published);

        let mut data = None;
        let mut encryption_key = None;

        if let Some(bytes) = self.data {
            if self.encrypted {
                let encrypted = EncryptedData::encrypt_aes(&bytes)?;
                data = Some(Data::Encrypted(encrypted.data));
                encryption_key = Some(encrypted.key);
            } else {
                let encoded = URL_SAFE_NO_PAD.encode(bytes);
                data = Some(Data::Base64(encoded));
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
            record_id: self.record_id.unwrap_or_default(),
        };

        let entry_id = msg.generate_record_id()?;

        if msg.record_id.is_empty() {
            msg.record_id = entry_id.clone();
        }

        msg.sign(
            self.actor.attestation.kid.clone(),
            &self.actor.attestation.jwk,
        )?;
        msg.authorize(
            self.actor.authorization.kid.clone(),
            &self.actor.authorization.jwk,
        )?;

        let reply = self.actor.dwn.process_message(msg).await?;

        match reply {
            Reply::Status(reply) => Ok(WriteResult {
                encryption_key,
                entry_id,
                reply,
            }),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }
}

pub struct WriteResult {
    /// If the data was encrypted, this is the generated encryption key.
    pub encryption_key: Option<Vec<u8>>,
    pub entry_id: String,
    pub reply: StatusReply,
}
