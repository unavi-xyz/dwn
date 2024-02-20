use crate::{
    handlers::{HandlerError, MethodHandler, Reply, Status, StatusReply},
    message::Message,
    store::{DataStore, MessageStore},
};

pub struct RecordsReadHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsReadHandler<'_, D, M> {
    async fn handle(
        &self,
        _tenant: &str,
        _message: Message,
    ) -> Result<impl Into<Reply>, HandlerError> {
        Ok(StatusReply {
            status: Status::ok(),
        })
    }
}
