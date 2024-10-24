use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use dwn_core::message::{
    cid::{compute_cid_cbor, CidGenerationError},
    Attestation, Header, Message, Signature,
};
use jose_jwk::jose_jwa::Signing;
use thiserror::Error;
use xdid::{
    core::{did::Did, did_url::DidUrl},
    methods::key::Signer,
};

use crate::Dwn;

pub struct Actor {
    pub did: Did,
    pub auth_key: Option<DocumentKey>,
    pub sign_key: Option<DocumentKey>,
    /// Local DWN to interact with.
    pub dwn: Option<Dwn>,
    /// URL of remote DWN to interact with.
    pub remote: Option<String>,
}

/// A key that is stored in the DID document.
pub struct DocumentKey {
    alg: Signing,
    key: Box<dyn Signer>,
    /// URL to the key.
    url: DidUrl,
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

    pub fn sign_message(&self, msg: &mut Message) -> Result<(), SignError> {
        let Some(doc_key) = self.sign_key.as_ref() else {
            return Err(SignError::NoKey);
        };

        let header = Header {
            alg: doc_key.alg,
            kid: doc_key.url.clone(),
        };

        let cid = compute_cid_cbor(&msg.descriptor)?;

        let header_str = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
        let payload = BASE64_URL_SAFE_NO_PAD.encode(cid);
        let input = header_str + "." + &payload;

        let signature = doc_key.key.sign(input.as_bytes())?;

        msg.attestation = Some(Attestation {
            payload,
            signatures: vec![Signature {
                header,
                signature: BASE64_URL_SAFE_NO_PAD.encode(signature),
            }],
        });

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum SignError {
    #[error("no signing key set")]
    NoKey,
    #[error(transparent)]
    CidGeneration(#[from] CidGenerationError),
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
        actor.sign_key = Some(DocumentKey {
            key: Box::new(key),
            alg: Signing::Ps256,
            url: DidUrl {
                did,
                path_abempty: "".to_string(),
                query: None,
                fragment: None,
            },
        });

        let mut msg = RecordsWriteBuilder::default().build().unwrap();
        actor.sign_message(&mut msg).unwrap();
        assert!(msg.attestation.is_some());
    }
}
