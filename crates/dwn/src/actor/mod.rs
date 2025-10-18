use std::sync::Arc;

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use dwn_core::message::{
    AuthPayload, Header, Jws, Message, Signature,
    cid::{CidGenerationError, compute_cid_cbor},
};
use reqwest::Url;
use thiserror::Error;
use xdid::{core::did::Did, methods::key::Signer};

use crate::Dwn;

use self::document_key::DocumentKey;

pub mod document_key;
pub mod protocols;
pub mod records;
pub mod sync;

#[derive(Clone)]
pub struct Actor {
    pub did: Did,
    pub dwn: Dwn,

    pub auth_key: Option<Arc<DocumentKey>>,
    pub sign_key: Option<Arc<DocumentKey>>,

    /// URL of a remote DWN to sync with.
    pub remote: Option<Url>,
    client: reqwest::Client,
}

impl Actor {
    pub fn new(did: Did, dwn: Dwn) -> Self {
        Self {
            did,
            dwn,
            auth_key: None,
            sign_key: None,
            remote: None,
            client: reqwest::Client::default(),
        }
    }

    /// Signs the message with a [DID assertion](https://www.w3.org/TR/did-core/#assertion) key.
    pub fn sign(&self, msg: &mut Message) -> Result<(), SignError> {
        let Some(doc_key) = self.sign_key.as_ref() else {
            return Err(SignError::MissingKey);
        };

        let header = Header {
            alg: doc_key.alg,
            kid: doc_key.url.clone(),
        };

        let cid = compute_cid_cbor(&msg.descriptor)?;

        let signature = sign_jws(doc_key.key.as_ref(), &header, &cid)?;

        msg.attestation = Some(Jws {
            payload: BASE64_URL_SAFE_NO_PAD.encode(cid),
            signatures: vec![Signature {
                header,
                signature: BASE64_URL_SAFE_NO_PAD.encode(signature),
            }],
        });

        Ok(())
    }

    /// Authorizes the message with a [DID authentication](https://www.w3.org/TR/did-core/#authentication) key.
    /// If the message has been signed, the assertion will also be authorized.
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

        let signature = sign_jws(doc_key.key.as_ref(), &header, &auth_payload)?;

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

fn sign_jws(key: &dyn Signer, header: &Header, payload: &str) -> Result<Vec<u8>, SignError> {
    let header_str = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
    let payload = BASE64_URL_SAFE_NO_PAD.encode(payload);
    let input = header_str + "." + &payload;
    let signature = key.sign(input.as_bytes())?;
    Ok(signature)
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
    use dwn_core::message::descriptor::RecordsWriteBuilder;
    use dwn_native_db::NativeDbStore;
    use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

    use super::*;

    #[test]
    fn test_sign() {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();

        let dwn = Dwn::from(NativeDbStore::new_in_memory().unwrap());
        let mut actor = Actor::new(did, dwn);
        actor.sign_key = Some(Arc::new(key.into()));

        let mut msg = RecordsWriteBuilder::default().build().unwrap();
        actor.sign(&mut msg).unwrap();
        assert!(msg.attestation.is_some());
    }
}
