use iana_media_types::MediaType;

use crate::{
    handlers::{RecordsQueryReply, Reply},
    message::{
        descriptor::{Filter, FilterDateCreated, FilterDateSort, RecordsQuery},
        Message,
    },
    store::{DataStore, MessageStore},
};

use super::{Actor, MessageSendError};

pub struct RecordsQueryBuilder<'a, D: DataStore, M: MessageStore> {
    actor: &'a Actor<D, M>,
    filter: Filter,
}

impl<'a, D: DataStore, M: MessageStore> RecordsQueryBuilder<'a, D, M> {
    pub fn new(actor: &'a Actor<D, M>) -> Self {
        RecordsQueryBuilder {
            actor,
            filter: Filter::default(),
        }
    }

    /// Filter by attester.
    pub fn attester(mut self, attester: String) -> Self {
        self.filter.attester = Some(attester);
        self
    }

    /// Filter by context ID.
    pub fn context_id(mut self, context_id: String) -> Self {
        self.filter.context_id = Some(context_id);
        self
    }

    /// Filter by data format.
    pub fn data_format(mut self, data_format: MediaType) -> Self {
        self.filter.data_format = Some(data_format);
        self
    }

    /// Sort by date.
    pub fn date_sort(mut self, date_sort: FilterDateSort) -> Self {
        self.filter.date_sort = Some(date_sort);
        self
    }

    /// Filter by message timestamp.
    pub fn message_timestamp(mut self, message_timestamp: FilterDateCreated) -> Self {
        self.filter.message_timestamp = Some(message_timestamp);
        self
    }

    /// Filter by parent ID.
    pub fn parent_id(mut self, parent_id: String) -> Self {
        self.filter.parent_id = Some(parent_id);
        self
    }

    /// Filter by protocol.
    pub fn protocol(mut self, protocol: String) -> Self {
        self.filter.protocol = Some(protocol);
        self
    }

    /// Filter by protocol version.
    pub fn protocol_version(mut self, protocol_version: String) -> Self {
        self.filter.protocol_version = Some(protocol_version);
        self
    }

    /// Filter by recipient.
    pub fn recipient(mut self, recipient: String) -> Self {
        self.filter.recipient = Some(recipient);
        self
    }

    /// Filter by record ID.
    pub fn record_id(mut self, record_id: String) -> Self {
        self.filter.record_id = Some(record_id);
        self
    }

    /// Filter by schema.
    pub fn schema(mut self, schema: String) -> Self {
        self.filter.schema = Some(schema);
        self
    }

    /// Send the message to the DWN.
    pub async fn send(self) -> Result<RecordsQueryReply, MessageSendError> {
        let mut msg = Message::new(RecordsQuery::new(self.filter));

        if msg.record_id.is_empty() {
            msg.record_id = msg.generate_record_id()?;
        }

        msg.authorize(self.actor.auth.kid.clone(), &self.actor.auth.jwk)?;

        let reply = self.actor.dwn.process_message(msg).await?;

        match reply {
            Reply::RecordsQuery(reply) => Ok(reply),
            _ => Err(MessageSendError::InvalidReply(reply)),
        }
    }
}
