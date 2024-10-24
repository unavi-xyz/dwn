use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use dwn_core::message::{
    cid::{compute_cid_cbor, CidGenerationError},
    Header, Jws, Message, Signature,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;
use xdid::{core::did::Did, methods::key::Signer};

use crate::Dwn;

use self::document_key::DocumentKey;

pub mod document_key;

pub struct Actor {
    pub did: Did,
    pub auth_key: Option<DocumentKey>,
    pub sign_key: Option<DocumentKey>,
    /// Local DWN to interact with.
    pub dwn: Option<Dwn>,
    /// URL of remote DWN to interact with.
    pub remote: Option<String>,
}

impl Actor {
    pub fn new(did: Did) -> Self {
        Self {
            did,
            auth_key: None,
            sign_key: None,
            dwn: None,
            remote: None,
        }
    }

    pub fn sign(&self, msg: &mut Message) -> Result<(), SignError> {
        let Some(doc_key) = self.sign_key.as_ref() else {
            return Err(SignError::MissingKey);
        };

        let header = Header {
            alg: doc_key.alg,
            kid: doc_key.url.clone(),
        };

        let cid = compute_cid_cbor(&msg.descriptor)?;

        let signature = sign_jws(&doc_key.key, &header, &cid)?;

        msg.attestation = Some(Jws {
            payload: BASE64_URL_SAFE_NO_PAD.encode(cid),
            signatures: vec![Signature {
                header,
                signature: BASE64_URL_SAFE_NO_PAD.encode(signature),
            }],
        });

        Ok(())
    }

    pub fn authorize(&self, msg: &mut Message) -> Result<(), SignError> {
        let Some(doc_key) = self.auth_key.as_ref() else {
            return Err(SignError::MissingKey);
        };

        let header = Header {
            alg: doc_key.alg,
            kid: doc_key.url.clone(),
        };

        let descriptor_cid = compute_cid_cbor(&msg.descriptor)?;

        let attestation_cid = match &msg.attestation {
            // Spec says to use the "attestation string"... but I don't know what
            // that means, so we use the attestation payload.
            Some(v) => Some(compute_cid_cbor(&v.payload)?),
            None => None,
        };

        let auth_payload = serde_json::to_string(&AuthPayload {
            descriptor_cid,
            permissions_grant_cid: None,
            attestation_cid,
        })
        .unwrap();

        let signature = sign_jws(&doc_key.key, &header, &auth_payload)?;

        msg.authorization = Some(Jws {
            payload: BASE64_URL_SAFE_NO_PAD.encode(auth_payload),
            signatures: vec![Signature {
                header,
                signature: BASE64_URL_SAFE_NO_PAD.encode(signature),
            }],
        });

        Ok(())
    }
}

fn sign_jws(key: &Box<dyn Signer>, header: &Header, payload: &str) -> Result<Vec<u8>, SignError> {
    let header_str = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
    let payload = BASE64_URL_SAFE_NO_PAD.encode(payload);
    let input = header_str + "." + &payload;
    let signature = key.sign(input.as_bytes())?;
    Ok(signature)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
struct AuthPayload {
    descriptor_cid: String,
    permissions_grant_cid: Option<String>,
    attestation_cid: Option<String>,
}

#[derive(Error, Debug)]
pub enum SignError {
    #[error(transparent)]
    CidGeneration(#[from] CidGenerationError),
    #[error("missing signing key")]
    MissingKey,
    #[error("failed to sign message")]
    Sign(#[from] xdid::methods::key::SignError),
}

#[cfg(test)]
mod tests {
    use xdid::methods::key::{p256::P256KeyPair, DidKeyPair, PublicKey};

    use crate::builders::records::write::RecordsWriteBuilder;

    use super::*;

    #[test]
    fn test_sign() {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();

        let mut actor = Actor::new(did.clone());
        actor.sign_key = Some(key.into());

        let mut msg = RecordsWriteBuilder::default().build().unwrap();
        actor.sign(&mut msg).unwrap();
        assert!(msg.attestation.is_some());
    }
}
