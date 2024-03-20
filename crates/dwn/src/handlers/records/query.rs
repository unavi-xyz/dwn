use crate::{
    handlers::{HandlerError, MethodHandler, RecordsQueryReply, Reply, Status},
    message::{descriptor::Descriptor, Message},
    store::{DataStore, MessageStore},
};

pub struct RecordsQueryHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsQueryHandler<'_, D, M> {
    async fn handle(&self, message: Message) -> Result<impl Into<Reply>, HandlerError> {
        let tenant = message.tenant().await;

        let filter = match message.descriptor {
            Descriptor::RecordsQuery(descriptor) => descriptor.filter,
            _ => {
                return Err(HandlerError::InvalidDescriptor(
                    "Not a RecordsQuery message".to_string(),
                ))
            }
        };

        let entries = self
            .message_store
            .query(tenant, filter.unwrap_or_default())
            .await?;

        Ok(RecordsQueryReply {
            entries,
            status: Status::ok(),
        })
    }
}
