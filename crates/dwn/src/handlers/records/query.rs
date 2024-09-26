use std::sync::Arc;

use crate::{
    message::{descriptor::Descriptor, DwnRequest},
    reply::{MessageReply, QueryReply, Status},
    store::MessageStore,
    HandleMessageError,
};

pub async fn handle_records_query(
    message_store: &Arc<dyn MessageStore>,
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
            message.author(),
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
