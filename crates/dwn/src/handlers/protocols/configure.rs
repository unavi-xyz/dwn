use std::sync::Arc;

use crate::{
    message::{
        descriptor::{protocols::ActionWho, Descriptor},
        DwnRequest,
    },
    reply::{MessageReply, Status, StatusReply},
    store::{DataStore, MessageStore},
    HandleMessageError,
};

pub async fn handle_protocols_configure(
    data_store: Arc<dyn DataStore>,
    message_store: &Arc<dyn MessageStore>,
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

    if let Some(definition) = &descriptor.definition {
        for structure in definition.structure.values() {
            for action in &structure.actions {
                if action.who == ActionWho::Anyone && action.of.is_some() {
                    return Err(HandleMessageError::InvalidDescriptor(
                        "Action 'of' is not allowed with 'Anyone'".to_string(),
                    ));
                };
            }
        }
    };

    if let Some(_cid) = &descriptor.last_configuration {
        todo!("Check if the last configuration is valid");
    }

    message_store
        .put(authorized, target, message, data_store.clone())
        .await?;

    Ok(StatusReply {
        status: Status::ok(),
    }
    .into())
}
