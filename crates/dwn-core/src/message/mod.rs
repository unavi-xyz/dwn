use jose_jwa::Signing;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub use mime;
pub use semver::Version;
pub use time::OffsetDateTime;
use xdid::core::did_url::DidUrl;

pub mod cid;
pub mod data;
pub mod descriptor;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub record_id: String,
    pub data: Option<data::Data>,
    pub descriptor: descriptor::Descriptor,
    pub attestation: Option<Jws>,
    pub authorization: Option<Jws>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Jws {
    /// Base64 encoded payload.
    pub payload: String,
    pub signatures: Vec<Signature>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub header: Header,
    /// Base64 encoded signature.
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub alg: Signing,
    pub kid: DidUrl,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthPayload {
    pub descriptor_cid: String,
    pub permissions_grant_cid: Option<String>,
    pub attestation_cid: Option<String>,
}
