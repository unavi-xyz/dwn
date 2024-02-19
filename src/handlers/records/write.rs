use crate::{
    handlers::{HandlerError, MessageReply, MethodHandler, Status},
    message::{descriptor::Descriptor, Message},
    store::{DataStore, MessageStore},
};

pub struct RecordsWriteHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsWriteHandler<'_, D, M> {
    async fn handle(&self, tenant: &str, message: Message) -> Result<MessageReply, HandlerError> {
        message.verify_auth().await?;

        let _descriptor = match &message.descriptor {
            Descriptor::RecordsWrite(descriptor) => descriptor,
            _ => return Err(HandlerError::InvalidDescriptor),
        };

        // TODO: Get existing messages for the record.

        self.message_store.put(tenant, message).await?;

        // TODO: Store data

        Ok(MessageReply {
            status: Status::ok(),
        })
    }
}
