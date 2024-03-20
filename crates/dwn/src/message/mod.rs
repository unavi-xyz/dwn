use didkit::JWK;
use libipld_core::error::SerdeError;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

use crate::{
    message::auth::{AuthPayload, Protected, SignatureEntry, JWS},
    util::{encode_cbor, EncodeError},
};

use self::{auth::SignatureVerifyError, data::Data, descriptor::Descriptor};

pub mod auth;
pub mod data;
pub mod descriptor;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Request {
    pub messages: Vec<Message>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Message {
    pub attestation: Option<JWS<String>>,
    pub authorization: Option<JWS<AuthPayload>>,
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
pub enum DecodeError {
    #[error(transparent)]
    Serde(#[from] SerdeError),
    #[error(transparent)]
    Decode(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum VerifyAuthError {
    #[error("Authorization JWS missing")]
    AuthorizationMissing,
    #[error("Signature missing")]
    SignatureMissing,
    #[error(transparent)]
    SignatureVerify(#[from] SignatureVerifyError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl Message {
    pub fn new(descriptor: impl Into<Descriptor>) -> Self {
        Self {
            attestation: None,
            authorization: None,
            data: None,
            descriptor: descriptor.into(),
            record_id: "".to_string(),
        }
    }

    pub fn authorize(&mut self, key_id: String, jwk: &JWK) -> Result<(), AuthError> {
        let descriptor_cid = encode_cbor(&self.descriptor)?.cid().to_string();

        let payload = AuthPayload {
            attestation_cid: None,
            descriptor_cid,
            permissions_grant_cid: None,
        };
        let payload_ser = serde_json::to_string(&payload)?;

        let algorithm = jwk.algorithm.ok_or(AuthError::MissingAlgorithm)?;

        let signature = didkit::ssi::jws::encode_sign(algorithm, &payload_ser, jwk)?;

        let jws = JWS {
            payload,
            signatures: vec![SignatureEntry {
                protected: Protected { algorithm, key_id },
                signature,
            }],
        };

        self.authorization = Some(jws);

        Ok(())
    }

    pub fn generate_record_id(&self) -> Result<String, EncodeError> {
        RecordIdGenerator::generate(&self.descriptor)
    }

    /// Returns the first DID in the authorization JWS.
    pub async fn tenant(&self) -> Option<String> {
        match self
            .verify_auth()
            .await
            .map(|dids| dids.first().map(|did| did.to_string()))
        {
            Ok(t) => t,
            _ => None,
        }
    }

    /// Verifies the authorization JWS.
    /// Returns the DIDs of the keys used to sign the payload.
    pub async fn verify_auth(&self) -> Result<Vec<String>, VerifyAuthError> {
        let auth = self
            .authorization
            .as_ref()
            .ok_or(VerifyAuthError::AuthorizationMissing)?;

        if auth.signatures.is_empty() {
            return Err(VerifyAuthError::SignatureMissing);
        }

        let payload = serde_json::to_string(&auth.payload)?;
        let payload = payload.as_bytes();

        let mut dids = Vec::new();

        for entry in &auth.signatures {
            let did = entry.verify(payload).await?;
            dids.push(did);
        }

        Ok(dids)
    }
}

#[derive(Serialize)]
struct RecordIdGenerator {
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
}

impl RecordIdGenerator {
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
