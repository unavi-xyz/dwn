use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use didkit::JWK;
use thiserror::Error;
use tracing::info;

use crate::message::Message;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authorization JWS missing")]
    JwsMissing,
    #[error("Failed to decode JWS: {0}")]
    JwsDecode(#[from] base64::DecodeError),
    #[error("Failed to parse JWS: {0}")]
    JwsParse(#[from] serde_json::Error),
    #[error("Kid missing from JWS")]
    KidMissing,
}

pub async fn authenticate(message: &Message) -> Result<(), AuthError> {
    let auth = message
        .authorization
        .as_ref()
        .ok_or(AuthError::JwsMissing)?;

    let signatures = match &auth.signatures {
        Some(s) => s,
        None => return Ok(()),
    };

    signatures
        .iter()
        .try_for_each(|entry| -> Result<(), AuthError> {
            let protected = match &entry.protected {
                Some(p) => p,
                None => return Ok(()),
            };
            let protected = URL_SAFE_NO_PAD.decode(protected)?;
            let protected = serde_json::from_slice::<JWK>(&protected)?;

            info!("Kid: {:?}", protected.key_id);

            Ok(())
        })?;

    Ok(())
}
