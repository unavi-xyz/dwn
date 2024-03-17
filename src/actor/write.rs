use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use time::OffsetDateTime;

use crate::{
    handlers::{Reply, StatusReply},
    message::{descriptor::RecordsWrite, Data, Message},
    store::{DataStore, MessageStore},
};

use super::{Actor, MessageSendError};

pub struct RecordsWriteBuilder<'a, D: DataStore, M: MessageStore> {
    actor: &'a Actor<D, M>,
    data: Option<Vec<u8>>,
    parent_id: Option<String>,
    record_id: Option<String>,
    store: bool,
}

impl<'a, D: DataStore, M: MessageStore> RecordsWriteBuilder<'a, D, M> {
    pub fn new(actor: &'a Actor<D, M>) -> Self {
        RecordsWriteBuilder {
            actor,
            data: None,
            parent_id: None,
            record_id: None,
            store: true,
        }
    }

    /// Data to be written.
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Parent record ID.
    pub fn parent_id(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
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

        let data = self.data.map(|data| {
            let encoded = URL_SAFE_NO_PAD.encode(data);
            Data::Base64(encoded)
        });

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

        msg.authorize(self.actor.kid.clone(), &self.actor.jwk)?;

        let reply = self.actor.dwn.process_message(&self.actor.did, msg).await?;

        match reply {
            Reply::Status(reply) => Ok(WriteResult { entry_id, reply }),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }
}

pub struct WriteResult {
    pub entry_id: String,
    pub reply: StatusReply,
}
