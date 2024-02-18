use didkit::JWK;
use libipld::{Block, DefaultParams};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    error::SerdeError,
    ipld::Ipld,
    multihash::Code,
    serde::{from_ipld, to_ipld},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

use crate::message::auth::{AuthPayload, Protected, SignatureEntry, JWS};

use self::{auth::SignatureVerifyError, descriptor::Descriptor};

pub mod auth;
pub mod descriptor;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Message {
    pub attestation: Option<JWS<String>>,
    pub authorization: Option<JWS<AuthPayload>>,
    pub data: Option<Data>,
    pub descriptor: Descriptor,
    #[serde(rename = "recordId")]
    pub record_id: Option<String>,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing JWK algorithm")]
    MissingAlgorithm,
    #[error("Missing public key")]
    MissingPublicKey,
    #[error("Failed to encode descriptor: {0}")]
    Encode(#[from] EncodeError),
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
pub enum EncodeError {
    #[error("Failed to serialize to IPLD: {0}")]
    Serde(#[from] SerdeError),
    #[error("Failed to encode to CBOR: {0}")]
    Encode(#[from] anyhow::Error),
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
        let descriptor_cid = self.descriptor.encode_block()?.cid().to_string();

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

    /// Decodes a CBOR block -> message
    pub fn decode_block(block: Block<DefaultParams>) -> Result<Self, DecodeError> {
        let ipld = block.decode::<DagCborCodec, Ipld>()?;
        let msg = from_ipld(ipld)?;
        Ok(msg)
    }
    /// Encodes the message -> CBOR block
    pub fn encode_block(&self) -> Result<Block<DefaultParams>, EncodeError> {
        let ipld = to_ipld(self)?;
        let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &ipld)?;
        Ok(block)
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Data {
    Base64(String),
    Encrypted(EncryptedData),
}

impl Data {
    pub fn encode(&self) -> Result<Ipld, SerdeError> {
        to_ipld(self)
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
