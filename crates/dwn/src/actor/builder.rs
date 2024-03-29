use thiserror::Error;

use crate::{
    encode::EncodeError,
    handlers::MessageReply,
    message::{AuthError, DwnRequest, Message},
    store::{DataStore, MessageStore},
    HandleMessageError,
};

use super::{Actor, PrepareError};

pub trait MessageBuilder: Sized {
    fn get_actor(&self) -> &Actor<impl DataStore, impl MessageStore>;
    fn get_authorized(&self) -> bool;
    fn get_target(&self) -> Option<String>;

    /// Whether the message should be authorized.
    /// Defaults to true.
    fn authorized(self, authorized: bool) -> Self;

    /// Set the target DID.
    /// Defaults to the actor's DID.
    fn target(self, target: String) -> Self;

    /// Create the inital message.
    fn create_message(&mut self) -> Result<Message, PrepareError>;

    /// Build the message.
    fn build(&mut self) -> Result<Message, ProcessMessageError> {
        let mut message = self.create_message()?;

        if message.record_id.is_empty() {
            message.record_id = message.entry_id()?;
        }

        let actor = self.get_actor();
        let authorized = self.get_authorized();

        if authorized {
            message.authorize(actor.authorization.key_id.clone(), &actor.authorization.jwk)?;
        }

        self.post_build(&mut message)?;

        Ok(message)
    }

    /// Hook called after building the message.
    fn post_build(&mut self, _message: &mut Message) -> Result<(), PrepareError> {
        Ok(())
    }

    /// Process the message with the local DWN.
    #[allow(async_fn_in_trait)]
    async fn process(&mut self) -> Result<MessageReply, ProcessMessageError> {
        let message = self.build()?;

        let actor = self.get_actor();
        let target = self.get_target();

        let reply = if let Some(target) = target {
            let request = DwnRequest { message, target };
            actor.dwn.process_message(request).await?
        } else {
            actor.process_message(message).await?
        };

        Ok(reply)
    }
}

#[derive(Debug, Error)]
pub enum ProcessMessageError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    PrepareError(#[from] PrepareError),
    #[error(transparent)]
    HandleMessageError(#[from] HandleMessageError),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error("Invalid reply")]
    InvalidReply,
}
