use crate::{
    handlers::{RecordsReadReply, Reply},
    message::{descriptor::RecordsRead, Message},
    store::{DataStore, MessageStore},
};

use super::{
    builder::{MessageBuilder, ProcessMessageError},
    Actor, PrepareError,
};

pub struct RecordsReadBuilder<'a, D: DataStore, M: MessageStore> {
    actor: &'a Actor<D, M>,
    authorized: bool,
    record_id: String,
    target: Option<String>,
}

impl<'a, D: DataStore, M: MessageStore> MessageBuilder for RecordsReadBuilder<'a, D, M> {
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

    fn build(&mut self) -> Result<Message, PrepareError> {
        Ok(Message::new(RecordsRead::new(self.record_id.clone())))
    }
}

impl<'a, D: DataStore, M: MessageStore> RecordsReadBuilder<'a, D, M> {
    pub fn new(actor: &'a Actor<D, M>, record_id: String) -> Self {
        RecordsReadBuilder {
            actor,
            authorized: true,
            record_id,
            target: None,
        }
    }

    pub async fn process(&mut self) -> Result<RecordsReadReply, ProcessMessageError> {
        let reply = MessageBuilder::process(self).await?;

        let reply = match reply {
            Reply::RecordsRead(reply) => reply,
            _ => return Err(ProcessMessageError::InvalidReply),
        };

        Ok(reply)
    }
}
