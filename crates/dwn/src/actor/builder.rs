use didkit::{
    ssi::{did::ServiceEndpoint, vc::OneOrMany},
    Document, ResolutionInputMetadata, ResolutionMetadata, DID_METHODS,
};
use thiserror::Error;
use tracing::{debug, warn};

use crate::{
    encode::EncodeError,
    message::{AuthError, DwnRequest, Message},
    reply::MessageReply,
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

    /// Send the message to a DID's DWN.
    #[allow(async_fn_in_trait)]
    async fn send(&mut self, did: &str) -> Result<MessageReply, ProcessMessageError> {
        let message = self.build()?;

        let actor = self.get_actor();
        let target = self.get_target().unwrap_or_else(|| actor.did.clone());
        let request = DwnRequest { message, target };

        let dwn_url = match resolve_dwn(did).await? {
            Some(url) => url,
            None => {
                return Err(ProcessMessageError::ResolveDid(
                    ResolveDidError::Resolution(format!("Failed to resolve DWN for {}", did)),
                ));
            }
        };

        debug!("Resolved {} to DWN {}", did, dwn_url);

        let reply = actor.send(request, &dwn_url).await?;
        Ok(reply)
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
    ResolveDid(#[from] ResolveDidError),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error("Invalid reply")]
    InvalidReply,
}

const DWN_SERVICE_TYPE: &str = "DWN";

async fn resolve_dwn(did: &str) -> Result<Option<String>, ResolveDidError> {
    let document = resolve_did(did).await?;

    Ok(document.service.and_then(|services| {
        services.iter().find_map(|service| {
            let types = match &service.type_ {
                OneOrMany::One(t) => vec![t.to_owned()],
                OneOrMany::Many(t) => t.to_owned(),
            };

            if !types.iter().any(|t| t == DWN_SERVICE_TYPE) {
                return None;
            }

            let endpoint =
                service
                    .service_endpoint
                    .as_ref()
                    .and_then(|endpoint| match endpoint {
                        OneOrMany::One(e) => Some(e),
                        OneOrMany::Many(e) => e.first(),
                    })?;

            match endpoint {
                ServiceEndpoint::URI(uri) => Some(uri.to_owned()),
                ServiceEndpoint::Map(_) => {
                    warn!("DWN service endpoint is not a URI.");
                    None
                }
            }
        })
    }))
}

#[derive(Debug, Error)]
pub enum ResolveDidError {
    #[error("Failed to parse DID: {0}")]
    DidParse(&'static str),
    #[error("Failed to resolve DID: {0}")]
    Resolution(String),
}

async fn resolve_did(did: &str) -> Result<Document, ResolveDidError> {
    match DID_METHODS
        .get_method(did)
        .map_err(ResolveDidError::DidParse)?
        .to_resolver()
        .resolve(did, &ResolutionInputMetadata::default())
        .await
    {
        (
            ResolutionMetadata {
                error: Some(err), ..
            },
            _,
            _,
        ) => Err(ResolveDidError::Resolution(err)),
        (_, Some(doc), _) => Ok(doc),
        _ => Err(ResolveDidError::Resolution("Unexpected result".to_string())),
    }
}
