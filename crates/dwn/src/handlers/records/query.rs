use crate::{
    handlers::{RecordsQueryReply, Reply, Status},
    message::{descriptor::Descriptor, Request},
    store::MessageStore,
    HandleMessageError,
};

pub async fn handle_records_query(
    message_store: &impl MessageStore,
    Request { target, message }: Request,
) -> Result<Reply, HandleMessageError> {
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
