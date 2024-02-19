use didkit::JWK;
use libipld::{pb::DagPbCodec, Cid};
use libipld_core::{
    codec::Codec,
    error::SerdeError,
    multihash::{Code, MultihashDigest},
    serde::to_ipld,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

use crate::{
    message::auth::{AuthPayload, Protected, SignatureEntry, JWS},
    util::{encode_cbor, CborEncodeError},
};

use self::{auth::SignatureVerifyError, descriptor::Descriptor};

pub mod auth;
pub mod builder;
pub mod descriptor;

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
    #[error("Failed to encode descriptor: {0}")]
    Encode(#[from] CborEncodeError),
    #[error("Failed to encode signature: {0}")]
    EncodeSignature(#[from] didkit::ssi::jws::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Failed to serialize to IPLD: {0}")]
    Serde(#[from] SerdeError),
    #[error("Failed to decode CBOR: {0}")]
    Decode(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum VerifyAuthError {
    #[error("Authorization JWS missing")]
    AuthorizationMissing,
    #[error("Signature missing")]
    SignatureMissing,
    #[error("Failed to verify signature: {0}")]
    SignatureVerify(#[from] SignatureVerifyError),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl Message {
    pub fn authorize(&mut self, kid: String, jwk: &JWK) -> Result<(), AuthError> {
        let descriptor_cid = encode_cbor(&self.descriptor)?.cid().to_string();

        let payload = AuthPayload {
            attestation_cid: None,
            descriptor_cid,
            permissions_grant_cid: None,
        };
        let payload_ser = serde_json::to_string(&payload)?;

        let alg = jwk.algorithm.ok_or(AuthError::MissingAlgorithm)?;

        let signature = didkit::ssi::jws::encode_sign(alg, &payload_ser, jwk)?;

        let jws = JWS {
            payload,
            signatures: vec![SignatureEntry {
                protected: Protected { alg, kid },
                signature,
            }],
        };

        self.authorization = Some(jws);

        Ok(())
    }

    pub fn generate_record_id(&self) -> Result<String, CborEncodeError> {
        RecordIdGenerator::generate(&self.descriptor)
    }

    pub async fn verify_auth(&self) -> Result<(), VerifyAuthError> {
        let auth = self
            .authorization
            .as_ref()
            .ok_or(VerifyAuthError::AuthorizationMissing)?;

        if auth.signatures.is_empty() {
            return Err(VerifyAuthError::SignatureMissing);
        }

        let payload = serde_json::to_string(&auth.payload)?;
        let payload = payload.as_bytes();

        for entry in &auth.signatures {
            entry.verify(payload).await?;
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct RecordIdGenerator {
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
}

impl RecordIdGenerator {
    pub fn generate(descriptor: &Descriptor) -> Result<String, CborEncodeError> {
        let generator = Self {
            descriptor_cid: encode_cbor(&descriptor)?.cid().to_string(),
        };
        encode_cbor(&generator).map(|block| block.cid().to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Data {
    Base64(String),
    Encrypted(EncryptedData),
}

impl Data {
    /// Returns the CID of the data after DAG-PB encoding.
    pub fn cid(&self) -> Result<Cid, CborEncodeError> {
        match self {
            Data::Base64(data) => {
                let ipld = to_ipld(data)?;
                let bytes = DagPbCodec.encode(&ipld)?;
                let hash = Code::Sha2_256.digest(&bytes);
                Ok(Cid::new_v1(DagPbCodec.into(), hash))
            }
            Data::Encrypted(_data) => {
                todo!()
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EncryptedData {
    pub protected: String,
    pub recipients: Vec<String>,
    pub ciphertext: String,
    pub iv: String,
    pub tag: String,
}
