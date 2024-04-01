use crate::{
    handlers::{MessageReply, QueryReply, Status},
    message::{descriptor::Descriptor, DwnRequest},
    store::MessageStore,
    HandleMessageError,
};

pub async fn handle_records_query(
    message_store: &impl MessageStore,
    DwnRequest {
        target,
        mut message,
    }: DwnRequest,
) -> Result<MessageReply, HandleMessageError> {
    let authorized = message.is_authorized(&target).await;

    let filter = match &mut message.descriptor {
        Descriptor::RecordsQuery(descriptor) => descriptor.filter.take(),
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a RecordsQuery message".to_string(),
            ))
        }
    };

    let entries = message_store
        .query_records(
            target,
            message.author().as_deref(),
            authorized,
            filter.unwrap_or_default(),
        )
        .await?;

    Ok(QueryReply {
        entries,
        status: Status::ok(),
    }
    .into())
}
