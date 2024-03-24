use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use openssl::{error::ErrorStack, rand::rand_bytes};
use time::OffsetDateTime;

use crate::{
    message::{
        data::{Data, EncryptedData},
        descriptor::RecordsWrite,
        Message,
    },
    store::{DataStore, MessageStore},
};

use super::{Actor, MessageSendError};

pub struct CreateRecord<'a> {
    /// Whether the message should be authorized.
    pub authorized: bool,
    /// Data to be written.
    pub data: Option<Vec<u8>>,
    /// Encryption to use on the data.
    pub encryption: Option<&'a Encryption>,
    /// Whether the record should be published.
    /// This makes the record public for anyone to read.
    pub published: bool,
    /// Whether the message should be signed.
    pub signed: bool,
}

impl Default for CreateRecord<'_> {
    fn default() -> Self {
        CreateRecord {
            authorized: true,
            data: None,
            encryption: None,
            published: false,
            signed: true,
        }
    }
}

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

impl CreateRecord<'_> {
    pub fn build<D: DataStore, M: MessageStore>(
        self,
        actor: &Actor<D, M>,
    ) -> Result<Message, MessageSendError> {
        build_write(actor, self, None, None)
    }
}

pub(crate) fn build_write<D: DataStore, M: MessageStore>(
    actor: &Actor<D, M>,
    create: CreateRecord,
    parent_id: Option<String>,
    record_id: Option<String>,
) -> Result<Message, MessageSendError> {
    let mut descriptor = RecordsWrite::default();
    descriptor.message_timestamp = OffsetDateTime::now_utc();
    descriptor.parent_id = parent_id;
    descriptor.published = Some(create.published);

    let mut data = None;

    if let Some(bytes) = create.data {
        match create.encryption {
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
        record_id: record_id.unwrap_or_default(),
    };

    if msg.record_id.is_empty() {
        msg.record_id = msg.entry_id()?;
    }

    if create.signed {
        msg.sign(actor.attestation.kid.clone(), &actor.attestation.jwk)?;
    }

    if create.authorized {
        msg.authorize(actor.authorization.kid.clone(), &actor.authorization.jwk)?;
    }

    Ok(msg)
}
