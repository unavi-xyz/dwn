use jose_jwk::Jwk;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

use crate::{
    did::Did,
    did_url::{DidUrl, RelativeDidUrl},
};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde_as]
pub struct Document {
    pub id: Did,
    pub also_known_as: Option<Vec<String>>,
    #[serde_as(as = "Option<OneOrMany<_>>")]
    pub controller: Option<Vec<Did>>,
    pub verification_method: Option<Vec<VerificationMethodMap>>,
    pub authentication: Option<Vec<VerificationMethod>>,
    pub assertion_method: Option<Vec<VerificationMethod>>,
    pub key_agreement: Option<Vec<VerificationMethod>>,
    pub capability_invocation: Option<Vec<VerificationMethod>>,
    pub capability_delegation: Option<Vec<VerificationMethod>>,
    pub service: Option<Vec<ServiceEndpoint>>,
}

impl Document {
    /// Returns the verification method that the provided [DidUrl] is
    /// referencing, restricted to a given [VerificationRole].
    pub fn resolve_verification_method(
        &self,
        url: &DidUrl,
        role: VerificationRole,
    ) -> Option<VerificationMethodMap> {
        let methods = match role {
            VerificationRole::Assertion => self.assertion_method.as_deref(),
            VerificationRole::Authentication => self.authentication.as_deref(),
            VerificationRole::CapabilityDelegation => self.capability_delegation.as_deref(),
            VerificationRole::CapabilityInvocation => self.capability_invocation.as_deref(),
            VerificationRole::KeyAgreement => self.key_agreement.as_deref(),
        }
        .unwrap_or_default();

        for method in methods {
            match method {
                VerificationMethod::Map(map) => {
                    if map.id == *url {
                        return Some(*map.clone());
                    }
                }
                VerificationMethod::RelativeUrl(relative_url) => {
                    return self.resolve_relative_url(relative_url);
                }
                VerificationMethod::Url(method_url) => {
                    if method_url.did == url.did {
                        if let Some(relative_url) = method_url.to_relative() {
                            return self.resolve_relative_url(&relative_url);
                        }
                    } else {
                        // TODO: Support additional DID resolution?
                    }
                }
            }
        }

        None
    }

    fn resolve_relative_url(&self, url: &RelativeDidUrl) -> Option<VerificationMethodMap> {
        for method in self.verification_method.as_deref().unwrap_or_default() {
            if method.id.to_relative().as_ref() == Some(url) {
                return Some(method.clone());
            }
        }

        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VerificationRole {
    Assertion,
    Authentication,
    CapabilityDelegation,
    CapabilityInvocation,
    KeyAgreement,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum VerificationMethod {
    Map(Box<VerificationMethodMap>),
    RelativeUrl(RelativeDidUrl),
    Url(DidUrl),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VerificationMethodMap {
    pub id: DidUrl,
    pub controller: Did,
    #[serde(rename = "type")]
    pub typ: String,
    pub public_key_jwk: Option<Jwk>,
    /// Multibase encoded public key.
    pub public_key_multibase: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde_as]
pub struct ServiceEndpoint {
    pub id: String,
    #[serde(rename = "type")]
    #[serde_as(as = "OneOrMany<_>")]
    pub typ: Vec<String>,
}
