use dwn_core::message::{Message, cid::CidGenerationError, descriptor::Descriptor};
use thiserror::Error;
use xdid::core::{ResolutionError, did::Did};

mod attestation;
mod authorization;
mod jws;

pub struct ValidationResult {
    /// DIDs with valid attestation signatures.
    pub _attested: Vec<Did>,
    /// DIDs with valid authentication signatures.
    pub authenticated: Vec<Did>,
}

pub async fn validate_message(msg: &Message) -> Result<ValidationResult, ValidationError> {
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

    let attested = if msg.attestation.is_some() {
        attestation::validate_attestation(msg).await?
    } else {
        Vec::new()
    };

    let authenticated = if msg.authorization.is_some() {
        authorization::validate_authorization(msg).await?
    } else {
        Vec::new()
    };

    Ok(ValidationResult {
        _attested: attested,
        authenticated,
    })
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
