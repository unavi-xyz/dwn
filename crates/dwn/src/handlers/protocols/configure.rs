use crate::{
    handlers::{MessageReply, Status, StatusReply},
    message::{descriptor::Descriptor, DwnRequest},
    store::{DataStore, MessageStore},
    HandleMessageError,
};

pub async fn handle_protocols_configure(
    data_store: &impl DataStore,
    message_store: &impl MessageStore,
    DwnRequest { target, message }: DwnRequest,
) -> Result<MessageReply, HandleMessageError> {
    let authorized = message.is_authorized(&target).await;

    if !authorized {
        return Err(HandleMessageError::Unauthorized);
    }

    let descriptor = match &message.descriptor {
        Descriptor::ProtocolsConfigure(descriptor) => descriptor,
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a ProtocolsConfigure message".to_string(),
            ))
        }
    };

    if let Some(_cid) = &descriptor.last_configuration {
        todo!("Check if the last configuration is valid");
    }

    message_store.put(target, message, data_store).await?;

    Ok(StatusReply {
        status: Status::ok(),
    }
    .into())
}
