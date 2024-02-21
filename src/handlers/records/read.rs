use crate::{
    handlers::{HandlerError, MethodHandler, RecordsReadReply, Reply, Status},
    message::{
        descriptor::{Descriptor, Filter, FilterDateSort},
        Message,
    },
    store::{DataStore, MessageStore},
};

pub struct RecordsReadHandler<'a, D: DataStore, M: MessageStore> {
    pub data_store: &'a D,
    pub message_store: &'a M,
}

impl<D: DataStore, M: MessageStore> MethodHandler for RecordsReadHandler<'_, D, M> {
    async fn handle(
        &self,
        tenant: &str,
        message: Message,
    ) -> Result<impl Into<Reply>, HandlerError> {
        let descriptor = match &message.descriptor {
            Descriptor::RecordsRead(descriptor) => descriptor,
            _ => {
                return Err(HandlerError::InvalidDescriptor(
                    "Not a RecordsRead message".to_string(),
                ));
            }
        };

        let messages = self
            .message_store
            .query(
                tenant,
                Filter {
                    record_id: Some(descriptor.record_id.clone()),
                    date_sort: Some(FilterDateSort::CreatedDescending),
                    ..Default::default()
                },
            )
            .await?;

        tracing::info!("Messages: {:?}", messages);

        // Get the latest commit or delete message.
        let latest_commit_or_delete = messages.iter().find(|m| {
            matches!(
                m.descriptor,
                Descriptor::RecordsCommit(_) | Descriptor::RecordsDelete(_)
            )
        });

        // If no message was found, use the initial entry.
        let record =
            latest_commit_or_delete
                .or(messages.last())
                .ok_or(HandlerError::InvalidDescriptor(
                    "Record not found".to_string(),
                ))?;

        // TODO: Get data from data store.

        Ok(RecordsReadReply {
            data: Vec::new(),
            record: record.clone(),
            status: Status::ok(),
        })
    }
}
