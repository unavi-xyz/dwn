use dwn_core::message::{Message, cid::CidGenerationError, descriptor::Descriptor};
use thiserror::Error;
use xdid::core::{ResolutionError, did::Did};

mod attestation;
mod authorization;
mod jws;

pub async fn validate_message(target: &Did, msg: &Message) -> Result<(), ValidationError> {
    if let Descriptor::RecordsWrite(desc) = &msg.descriptor
        && msg.data.is_some()
    {
        if desc.data_cid.is_none() {
            return Err(ValidationError::MissingDataInfo);
        }

        if desc.data_format.is_none() {
            return Err(ValidationError::MissingDataInfo);
        }
    }

    if msg.attestation.is_some() {
        attestation::validate_attestation(target, msg).await?;
    }

    if msg.authorization.is_some() {
        authorization::validate_authorization(target, msg).await?;
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("failed to generate CID: {0}")]
    CidGeneration(#[from] CidGenerationError),
    #[error("failed to decode base64: {0}")]
    Decode(#[from] base64::DecodeError),
    #[error("failed to construct DID resolver: {0}")]
    DidResolver(#[from] xdid::resolver::MethodError),
    #[error("invalid kid")]
    InvalidKid,
    #[error("invalid payload")]
    InvalidPayload,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("missing data information")]
    MissingDataInfo,
    #[error("missing signature")]
    MissingSignature,
    #[error("Error during serialization / deserialization: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("failed to resolve DID: {0}")]
    ResolutionError(#[from] ResolutionError),
    #[error("unsupported algorithm")]
    UnsupportedAlgorithm,
    #[error("unsupported key")]
    UnsupportedKey,
}
