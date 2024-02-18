use std::str::FromStr;

use async_recursion::async_recursion;
use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};
use didkit::{
    ssi::{
        did::{VerificationMethod, VerificationMethodMap},
        jwk::Algorithm,
    },
    Document, ResolutionInputMetadata, DIDURL, DID_METHODS,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::skip_serializing_none;
use thiserror::Error;
use tracing::{debug};

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AuthPayload {
    #[serde(rename = "attestationCid")]
    pub attestation_cid: Option<String>,
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
    #[serde(rename = "permissionsGrantCid")]
    pub permissions_grant_cid: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct JWS<T> {
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
    #[error("DID error: {0}")]
    DidError(#[from] didkit::ssi::did::Error),
    #[error("JWS error: {0}")]
    JwsError(#[from] didkit::ssi::jws::Error),
    #[error("Unsupported DID method: {0}")]
    UnsupportedDidMethod(&'static str),
    #[error("DID resolution error: {0}")]
    ResolutionError(String),
    #[error("DID Document not found")]
    DocumentNotFound,
    #[error("Verification method not found")]
    MethodNotFound,
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl SignatureEntry {
    pub async fn verify(&self, payload: &[u8]) -> Result<(), SignatureVerifyError> {
        // Resolve DID URL
        let did_url = DIDURL::from_str(&self.protected.kid)?;
        let verification_method = resolve_method_map(&did_url).await?;

        // Verify signature
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

async fn resolve_did_url_document(did_url: &DIDURL) -> Result<Document, SignatureVerifyError> {
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

#[async_recursion]
async fn resolve_method_map(
    did_url: &DIDURL,
) -> Result<VerificationMethodMap, SignatureVerifyError> {
    let doc = resolve_did_url_document(did_url).await?;

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
            resolve_method_map(&DIDURL {
                did: did_url.did.clone(),
                query: url.query.clone(),
                fragment: url.fragment.clone(),
                path_abempty: url.path.to_string(),
            })
            .await?
        }
        VerificationMethod::DIDURL(url) => resolve_method_map(url).await?,
    };

    Ok(method_map)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Protected {
    /// Algorithm used to verify the signature
    pub alg: Algorithm,
    /// DID URL of the key used to verify the signature
    pub kid: String,
}

#[derive(Deserialize, Serialize)]
struct ProtectedJson {
    pub alg: Algorithm,
    pub kid: String,
}

impl<'de> Deserialize<'de> for Protected {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        let decoded = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(serde::de::Error::custom)?;
        let json =
            serde_json::from_slice::<ProtectedJson>(&decoded).map_err(serde::de::Error::custom)?;
        Ok(Protected {
            alg: json.alg,
            kid: json.kid,
        })
    }
}

impl Serialize for Protected {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json = ProtectedJson {
            alg: self.alg,
            kid: self.kid.clone(),
        };
        let json_string = serde_json::to_string(&json).map_err(serde::ser::Error::custom)?;
        let encoded = URL_SAFE_NO_PAD.encode(json_string);
        serializer.serialize_str(&encoded)
    }
}
