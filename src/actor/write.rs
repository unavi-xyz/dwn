use time::OffsetDateTime;

use crate::{
    handlers::{Reply, StatusReply},
    message::{descriptor::RecordsWrite, Data, Message},
    store::{DataStore, MessageStore},
};

use super::{Actor, MessageSendError};

pub struct RecordsWriteBuilder<'a, D: DataStore, M: MessageStore> {
    actor: &'a Actor<D, M>,
    data: Option<Data>,
    record_id: Option<String>,
    store: bool,
}

impl<'a, D: DataStore, M: MessageStore> RecordsWriteBuilder<'a, D, M> {
    pub fn new(actor: &'a Actor<D, M>) -> Self {
        RecordsWriteBuilder {
            actor,
            data: None,
            record_id: None,
            store: true,
        }
    }

    /// Data to be written.
    pub fn data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }

    /// Whether to store the message locally.
    /// Defaults to true.
    pub fn store(mut self, store: bool) -> Self {
        self.store = store;
        self
    }

    /// Record ID to write to.
    /// If not provided, a new record ID will be generated.
    pub fn record_id(mut self, record_id: String) -> Self {
        self.record_id = Some(record_id);
        self
    }

    /// Send the message to the DWN.
    pub async fn send(self) -> Result<WriteResult, MessageSendError> {
        let mut descriptor = RecordsWrite::default();
        descriptor.message_timestamp = OffsetDateTime::now_utc();

        if let Some(data) = &self.data {
            let cid = data.cid()?;
            descriptor.data_cid = Some(cid.to_string());
        }

        let mut msg = Message {
            attestation: None,
            authorization: None,
            data: self.data,
            descriptor: descriptor.into(),
            record_id: self.record_id.unwrap_or_default(),
        };

        if msg.record_id.is_empty() {
            msg.record_id = msg.generate_record_id()?;
        }

        let record_id = msg.record_id.clone();

        msg.authorize(self.actor.kid.clone(), &self.actor.jwk)?;

        let reply = self.actor.dwn.process_message(&self.actor.did, msg).await?;

        match reply {
            Reply::Status(reply) => Ok(WriteResult { record_id, reply }),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }
}

pub struct WriteResult {
    pub record_id: String,
    pub reply: StatusReply,
}
