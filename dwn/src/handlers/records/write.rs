use crate::{
    handlers::{auth::authenticate, HandlerError, MessageReply, MethodHandler, Status},
    message::Message,
    store::{DataStore, MessageStore},
};

pub struct RecordsWriteHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsWriteHandler<'_, D, M> {
    async fn handle(&self, tenant: &str, message: Message) -> Result<MessageReply, HandlerError> {
        authenticate(&message).await?;

        Ok(MessageReply {
            status: Status::ok(),
        })
    }
}
