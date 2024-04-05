use std::str::FromStr;

use async_recursion::async_recursion;
use didkit::{
    ssi::{
        did::{VerificationMethod, VerificationMethodMap},
        jwk::Algorithm,
        jws::Header,
    },
    Document, ResolutionInputMetadata, VerificationRelationship, DIDURL, DID_METHODS,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use tracing::debug;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AuthPayload {
    #[serde(rename = "attestationCid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation_cid: Option<String>,
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
    #[serde(rename = "permissionsGrantCid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_grant_cid: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Jws<T> {
    pub payload: T,
    pub signatures: Vec<SignatureEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SignatureEntry {
    pub protected: Protected,
    pub signature: String,
}

#[derive(Debug, Error)]
pub enum SignatureVerifyError {
    #[error(transparent)]
    DidError(#[from] didkit::ssi::did::Error),
    #[error(transparent)]
    JwsError(#[from] didkit::ssi::jws::Error),
    #[error("Unsupported DID method: {0}")]
    UnsupportedDidMethod(&'static str),
    #[error("DID resolution error: {0}")]
    ResolutionError(String),
    #[error("DID Document not found")]
    DocumentNotFound,
    #[error("Verification method not found")]
    MethodNotFound,
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl SignatureEntry {
    /// Verify the signature of the payload.
    pub async fn verify(
        &self,
        payload: &[u8],
        relationship: VerificationRelationship,
    ) -> Result<(), SignatureVerifyError> {
        // Resolve the key.
        let did_url = DIDURL::from_str(&self.protected.key_id)?;
        let verification_method = resolve_vc(&did_url, relationship).await?;

        // Verify signature.
        let jwk = verification_method.get_jwk()?;
        let (_, decoded) = didkit::ssi::jws::decode_verify(&self.signature, &jwk)?;

        if decoded != payload {
            return Err(SignatureVerifyError::JwsError(
                didkit::ssi::jws::Error::InvalidSignature,
            ));
        }

        Ok(())
    }
}

async fn resolve_vc(
    did_url: &DIDURL,
    relationship: VerificationRelationship,
) -> Result<VerificationMethodMap, SignatureVerifyError> {
    let doc = resolve_did_document(did_url).await?;

    let method_ids = doc
        .get_verification_method_ids(relationship)
        .map_err(|_| SignatureVerifyError::MethodNotFound)?;

    if !method_ids.contains(&did_url.to_string()) {
        return Err(SignatureVerifyError::MethodNotFound);
    }

    resolve_vc_map(did_url).await
}

#[async_recursion]
async fn resolve_vc_map(did_url: &DIDURL) -> Result<VerificationMethodMap, SignatureVerifyError> {
    let doc = resolve_did_document(did_url).await?;

    let verification_methods = doc
        .verification_method
        .ok_or(SignatureVerifyError::MethodNotFound)?;

    let method = verification_methods
        .iter()
        .find(|m| m.get_id(&did_url.did) == did_url.to_string())
        .ok_or(SignatureVerifyError::MethodNotFound)?;

    let method_map = match method {
        VerificationMethod::Map(m) => m.to_owned(),
        VerificationMethod::RelativeDIDURL(url) => {
            resolve_vc_map(&DIDURL {
                did: did_url.did.clone(),
                fragment: url.fragment.clone(),
                path_abempty: url.path.to_string(),
                query: url.query.clone(),
            })
            .await?
        }
        VerificationMethod::DIDURL(url) => resolve_vc_map(url).await?,
    };

    Ok(method_map)
}

async fn resolve_did_document(did_url: &DIDURL) -> Result<Document, SignatureVerifyError> {
    let did = did_url.did.clone();
    let did_method = DID_METHODS
        .get_method(&did)
        .map_err(SignatureVerifyError::UnsupportedDidMethod)?;
    let resolver = did_method.to_resolver();

    debug!("Resolving DID: {}", did);

    let input_metadata = ResolutionInputMetadata::default();
    let (resolution, doc, _) = resolver.resolve(&did, &input_metadata).await;

    if let Some(err) = resolution.error {
        return Err(SignatureVerifyError::ResolutionError(err));
    }

    let doc = doc.ok_or(SignatureVerifyError::DocumentNotFound)?;

    Ok(doc)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Protected {
    /// Algorithm used to verify the signature
    pub algorithm: Algorithm,
    /// DID URL of the VC used to verify the signature
    pub key_id: String,
}

impl<'de> Deserialize<'de> for Protected {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let header = Header::deserialize(deserializer)?;
        let key_id = header
            .key_id
            .ok_or(serde::de::Error::custom("key id is required"))?;
        Ok(Protected {
            algorithm: header.algorithm,
            key_id,
        })
    }
}

impl Serialize for Protected {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let header = Header {
            algorithm: self.algorithm,
            key_id: Some(self.key_id.clone()),
            ..Default::default()
        };
        header.serialize(serializer)
    }
}
