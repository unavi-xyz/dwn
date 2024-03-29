use crate::{
    handlers::{MessageReply, RecordsQueryReply, Status},
    message::{descriptor::Descriptor, DwnRequest},
    store::MessageStore,
    HandleMessageError,
};

pub async fn handle_records_query(
    message_store: &impl MessageStore,
    DwnRequest { target, message }: DwnRequest,
) -> Result<MessageReply, HandleMessageError> {
    let authorized = message.is_authorized(&target).await;

    let filter = match message.descriptor {
        Descriptor::RecordsQuery(descriptor) => descriptor.filter,
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a RecordsQuery message".to_string(),
            ))
        }
    };

    let entries = message_store
        .query(target, authorized, filter.unwrap_or_default())
        .await?;

    Ok(RecordsQueryReply {
        entries,
        status: Status::ok(),
    }
    .into())
}
