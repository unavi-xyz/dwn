use crate::{
    message::{descriptor::Descriptor, DwnRequest},
    reply::{MessageReply, QueryReply, Status},
    store::MessageStore,
    HandleMessageError,
};

pub async fn handle_protocols_query(
    message_store: &impl MessageStore,
    DwnRequest { target, message }: DwnRequest,
) -> Result<MessageReply, HandleMessageError> {
    let authorized = message.is_authorized(&target).await;

    let descriptor = match message.descriptor {
        Descriptor::ProtocolsQuery(descriptor) => descriptor,
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a ProtocolsQuery message".to_string(),
            ))
        }
    };

    let entries = message_store
        .query_protocols(target, authorized, descriptor.filter)
        .await?;

    Ok(QueryReply {
        entries,
        status: Status::ok(),
    }
    .into())
}
