use crate::{
    actor::{Actor, MessageBuilder, PrepareError, ProcessMessageError},
    message::{
        descriptor::records::{RecordsFilter, RecordsQuery},
        Message,
    },
    reply::{MessageReply, QueryReply},
    store::{DataStore, MessageStore},
};

pub struct RecordsQueryBuilder<'a, D: DataStore, M: MessageStore> {
    actor: &'a Actor<D, M>,
    authorized: bool,
    filter: RecordsFilter,
    target: Option<String>,
}

impl<'a, D: DataStore, M: MessageStore> MessageBuilder for RecordsQueryBuilder<'a, D, M> {
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

    fn create_message(&mut self) -> Result<Message, PrepareError> {
        Ok(Message::new(RecordsQuery::new(self.filter.clone())))
    }
}

impl<'a, D: DataStore, M: MessageStore> RecordsQueryBuilder<'a, D, M> {
    pub fn new(actor: &'a Actor<D, M>, filter: RecordsFilter) -> Self {
        RecordsQueryBuilder {
            actor,
            authorized: true,
            filter,
            target: None,
        }
    }

    pub async fn process(&mut self) -> Result<QueryReply, ProcessMessageError> {
        let reply = MessageBuilder::process(self).await?;

        let reply = match reply {
            MessageReply::Query(reply) => reply,
            _ => return Err(ProcessMessageError::InvalidReply),
        };

        Ok(reply)
    }

    pub async fn send(mut self, did: &str) -> Result<QueryReply, ProcessMessageError> {
        let reply = MessageBuilder::send(&mut self, did).await?;

        let reply = match reply {
            MessageReply::Query(reply) => reply,
            _ => return Err(ProcessMessageError::InvalidReply),
        };

        Ok(reply)
    }
}
