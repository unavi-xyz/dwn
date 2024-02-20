use crate::{
    handlers::{HandlerError, MethodHandler, Reply, Status},
    message::Message,
    store::{DataStore, MessageStore},
};

pub struct RecordsDeleteHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsDeleteHandler<'_, D, M> {
    async fn handle(&self, _tenant: &str, _message: Message) -> Result<Reply, HandlerError> {
        Ok(Reply::Status {
            status: Status::ok(),
        })
    }
}
