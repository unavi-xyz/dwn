use crate::{
    handlers::{RecordsQueryReply, Reply, Status},
    message::{descriptor::Descriptor, Message},
    store::MessageStore,
    HandleMessageError,
};

pub async fn handle_records_query(
    message_store: &impl MessageStore,
    message: Message,
) -> Result<Reply, HandleMessageError> {
    let tenant = message.tenant();

    let filter = match message.descriptor {
        Descriptor::RecordsQuery(descriptor) => descriptor.filter,
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a RecordsQuery message".to_string(),
            ))
        }
    };

    let entries = message_store
        .query(tenant, filter.unwrap_or_default())
        .await?;

    Ok(RecordsQueryReply {
        entries,
        status: Status::ok(),
    }
    .into())
}
