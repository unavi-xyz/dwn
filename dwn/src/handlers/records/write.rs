use crate::{
    handlers::{MessageReply, MethodHandler, Status},
    message::Message,
    store::{surrealdb::message::MessageStoreError, DataStore, MessageStore},
};

pub struct RecordsWriteHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsWriteHandler<'_, D, M> {
    type Error = MessageStoreError;

    fn handle(&self, tenant: &str, message: Message) -> Result<MessageReply, Self::Error> {
        Ok(MessageReply {
            status: Status::ok(),
        })
    }
}
