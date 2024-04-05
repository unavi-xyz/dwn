use std::str::FromStr;

use didkit::{VerificationRelationship, DIDURL, JWK};
use libipld_core::error::SerdeError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

use crate::{
    encode::{encode_cbor, EncodeError},
    message::auth::{AuthPayload, Jws, SignatureEntry},
};

use self::{auth::SignatureVerifyError, descriptor::Descriptor};

mod auth;
mod data;
pub mod descriptor;

pub use data::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DwnRequest {
    pub message: Message,
    /// Target DID.
    pub target: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation: Option<Jws<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<Jws<AuthPayload>>,
    #[serde(rename = "contextId", skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Data>,
    pub descriptor: Descriptor,
    #[serde(rename = "recordId")]
    pub record_id: String,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing JWK algorithm")]
    MissingAlgorithm,
    #[error("Missing public key")]
    MissingPublicKey,
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    EncodeSignature(#[from] didkit::ssi::jws::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum SignError {
    #[error("Missing JWK algorithm")]
    MissingAlgorithm,
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    EncodeSignature(#[from] didkit::ssi::jws::Error),
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error(transparent)]
    Serde(#[from] SerdeError),
    #[error(transparent)]
    Decode(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum ValidateError {
    #[error("JWS missing")]
    JwsMissing,
    #[error("Signature missing")]
    SignatureMissing,
    #[error(transparent)]
    SignatureVerify(#[from] SignatureVerifyError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("Invalid signature: {0}")]
    InvalidSignature(&'static str),
    #[error(transparent)]
    Encode(#[from] EncodeError),
}

impl Message {
    pub fn new(descriptor: impl Into<Descriptor>) -> Self {
        Self {
            attestation: None,
            authorization: None,
            context_id: None,
            data: None,
            descriptor: descriptor.into(),
            record_id: "".to_string(),
        }
    }

    pub fn authorize(&mut self, key_id: String, jwk: &JWK) -> Result<(), AuthError> {
        let descriptor_cid = encode_cbor(&self.descriptor)?.cid().to_string();

        let mut attestation_cid = None;

        if let Some(attestation) = &self.attestation {
            attestation_cid = Some(encode_cbor(&attestation.payload)?.cid().to_string());
        }

        let payload = AuthPayload {
            attestation_cid,
            descriptor_cid,
            permissions_grant_cid: None,
        };
        let payload_ser = serde_json::to_string(&payload)?;

        let algorithm = jwk.algorithm.ok_or(AuthError::MissingAlgorithm)?;

        let signature = didkit::ssi::jws::encode_sign(algorithm, &payload_ser, jwk)?;

        let jws = Jws {
            payload,
            signatures: vec![SignatureEntry {
                protected: auth::Protected { algorithm, key_id },
                signature,
            }],
        };

        self.authorization = Some(jws);

        Ok(())
    }

    /// Returns the author of the message.
    /// The author is the key used in the authorization JWS.
    pub fn author(&self) -> Option<String> {
        self.authorization
            .as_ref()
            .and_then(|jws| jws.signatures.first())
            .and_then(|entry| DIDURL::from_str(&entry.protected.key_id).ok())
            .map(|did_url| did_url.did)
    }

    pub fn entry_id(&self) -> Result<String, EncodeError> {
        EntryIdGenerator::generate(&self.descriptor)
    }

    pub fn sign(&mut self, key_id: String, jwk: &JWK) -> Result<(), SignError> {
        let payload = encode_cbor(&self.descriptor)?.cid().to_string();
        let algorithm = jwk.algorithm.ok_or(SignError::MissingAlgorithm)?;

        let signature = didkit::ssi::jws::encode_sign(algorithm, &payload, jwk)?;

        let jws = Jws {
            payload,
            signatures: vec![SignatureEntry {
                protected: auth::Protected { algorithm, key_id },
                signature,
            }],
        };

        self.attestation = Some(jws);

        Ok(())
    }

    /// Checks whether the key used in the authorization JWS is an authorization key for the given DID.
    pub async fn is_authorized(&self, did: &str) -> bool {
        if let Err(e) = self.validate_authorization().await {
            debug!("Failed to validate authorization JWS: {}", e);
            return false;
        }

        let jws = match &self.authorization {
            Some(jws) => jws,
            None => return false,
        };

        for signature in &jws.signatures {
            let did_url = match DIDURL::from_str(&signature.protected.key_id) {
                Ok(did_url) => did_url,
                Err(_) => continue,
            };

            if did_url.did == did {
                return true;
            }
        }

        debug!("No valid signature for DID {}", did);

        false
    }

    /// Validates the authorization JWS.
    pub async fn validate_authorization(&self) -> Result<(), ValidateError> {
        let jws = self
            .authorization
            .as_ref()
            .ok_or(ValidateError::JwsMissing)?;

        // Verify attestation CID matches
        if let Some(cid) = jws.payload.attestation_cid.as_ref() {
            let attestation = self
                .attestation
                .as_ref()
                .ok_or(ValidateError::InvalidSignature("no attestation"))?;

            let attestation_cid = encode_cbor(&attestation.payload)?.cid().to_string();

            if cid != &attestation_cid {
                return Err(ValidateError::InvalidSignature("attestation CID mismatch"));
            }
        } else if self.attestation.is_some() {
            return Err(ValidateError::InvalidSignature("attestation CID missing"));
        }

        // Verify descriptor CID matches
        let descriptor_cid = encode_cbor(&self.descriptor)?.cid().to_string();

        if jws.payload.descriptor_cid != descriptor_cid {
            return Err(ValidateError::InvalidSignature("descriptor CID mismatch"));
        }

        // Verify payload signature
        let payload = serde_json::to_string(&jws.payload)?;
        let payload = payload.as_bytes();

        verify_jws(jws, payload, VerificationRelationship::Authentication).await
    }

    /// Validates the attestation JWS.
    pub async fn validate_attestation(&self) -> Result<(), ValidateError> {
        let jws = self.attestation.as_ref().ok_or(ValidateError::JwsMissing)?;
        let payload = jws.payload.as_bytes();
        verify_jws(jws, payload, VerificationRelationship::AssertionMethod).await
    }
}

/// Verify the JWS signatures.
async fn verify_jws<T>(
    jws: &Jws<T>,
    payload: &[u8],
    relationship: VerificationRelationship,
) -> Result<(), ValidateError> {
    if jws.signatures.is_empty() {
        return Err(ValidateError::SignatureMissing);
    }

    for entry in &jws.signatures {
        entry.verify(payload, relationship.clone()).await?;
    }

    Ok(())
}

#[derive(Serialize)]
struct EntryIdGenerator {
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
}

impl EntryIdGenerator {
    pub fn generate(descriptor: &Descriptor) -> Result<String, EncodeError> {
        let generator = Self {
            descriptor_cid: encode_cbor(&descriptor)?.cid().to_string(),
        };
        encode_cbor(&generator).map(|block| block.cid().to_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = Message {
            attestation: None,
            authorization: None,
            context_id: None,
            data: Some(Data::Base64("hello".to_string())),
            descriptor: Descriptor::RecordsWrite(Default::default()),
            record_id: "world".to_string(),
        };

        let serialized = serde_json::to_string(&message).unwrap();

        let value: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(value["data"], "hello");
        assert_eq!(value["descriptor"]["interface"], "Records");
        assert_eq!(value["descriptor"]["method"], "Write");
        assert_eq!(value["recordId"], "world");

        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, deserialized);
    }
}
