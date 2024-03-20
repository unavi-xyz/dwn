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
        let mut tenant = None;

        if message.attestation.is_some() && message.authorization.is_some() {
            let dids = message.verify_attestation().await.unwrap();
            tenant = dids.first().map(|d| d.to_string());
        }

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
