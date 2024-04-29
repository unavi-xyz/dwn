use crate::{
    actor::{Actor, MessageBuilder, PrepareError, ProcessMessageError},
    message::{descriptor::records::RecordsDelete, Message},
    reply::{MessageReply, StatusReply},
    store::{DataStore, MessageStore},
};

pub struct RecordsDeleteBuilder<'a, D: DataStore, M: MessageStore> {
    actor: &'a Actor<D, M>,
    authorized: bool,
    record_id: String,
    target: Option<String>,

    final_entry_id: String,
}

impl<'a, D: DataStore, M: MessageStore> MessageBuilder for RecordsDeleteBuilder<'a, D, M> {
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
        self.final_entry_id = message.entry_id()?;
        Ok(())
    }

    fn create_message(&mut self) -> Result<Message, PrepareError> {
        let mut msg = Message::new(RecordsDelete::new(self.record_id.clone()));
        msg.record_id = self.record_id.clone();
        Ok(msg)
    }
}

impl<'a, D: DataStore, M: MessageStore> RecordsDeleteBuilder<'a, D, M> {
    pub fn new(actor: &'a Actor<D, M>, record_id: String) -> Self {
        RecordsDeleteBuilder {
            actor,
            authorized: true,
            final_entry_id: String::new(),
            record_id,
            target: None,
        }
    }

    pub async fn process(mut self) -> Result<DeleteResponse, ProcessMessageError> {
        let reply = MessageBuilder::process(&mut self).await?;

        let reply = match reply {
            MessageReply::Status(reply) => reply,
            _ => return Err(ProcessMessageError::InvalidReply),
        };

        Ok(DeleteResponse {
            entry_id: self.final_entry_id.clone(),
            reply,
        })
    }
}

pub struct DeleteResponse {
    pub entry_id: String,
    pub reply: StatusReply,
}
