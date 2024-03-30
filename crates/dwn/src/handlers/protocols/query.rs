use crate::{
    handlers::{MessageReply, Status, StatusReply},
    message::{descriptor::Descriptor, DwnRequest},
    store::{DataStore, MessageStore},
    HandleMessageError,
};

pub async fn handle_protocols_query(
    _data_store: &impl DataStore,
    _message_store: &impl MessageStore,
    DwnRequest { target, message }: DwnRequest,
) -> Result<MessageReply, HandleMessageError> {
    let authorized = message.is_authorized(&target).await;

    if !authorized {
        return Err(HandleMessageError::Unauthorized);
    }

    let _descriptor = match &message.descriptor {
        Descriptor::ProtocolsQuery(descriptor) => descriptor,
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a ProtocolsQuery message".to_string(),
            ))
        }
    };

    // TODO: Query protocols

    Ok(StatusReply {
        status: Status::ok(),
    }
    .into())
}
