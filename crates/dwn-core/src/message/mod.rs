use std::fmt::Display;

use jose_jwa::Signing;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};

pub use mime;
pub use semver::Version;
pub use time::OffsetDateTime;
use xdid::core::did_url::DidUrl;

pub mod cid;
pub mod data;
mod record_id;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct Message {
    pub record_id: String,
    pub context_id: Option<String>,
    pub data: Option<data::Data>,
    pub descriptor: Descriptor,
    pub attestation: Option<Attestation>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct Descriptor {
    pub interface: Interface,
    pub method: Method,
    pub data_cid: Option<String>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub data_format: Option<mime::Mime>,
    pub parent_id: Option<String>,
    pub protocol: Option<String>,
    pub protocol_version: Option<Version>,
    pub schema: Option<String>,
    pub published: Option<bool>,
    pub date_created: OffsetDateTime,
    pub date_published: Option<OffsetDateTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Interface {
    Permissions,
    Protocols,
    Records,
}

impl Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Read,
    Query,
    Write,
    Subscribe,
    Delete,
    Configure,
    Request,
    Grant,
    Revocation,
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Attestation {
    pub payload: String,
    pub signatures: Vec<Signature>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    // DWN spec says to use protected headers here... but then how would
    // you read them to verify the JWS?
    pub header: Header,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub alg: Signing,
    pub kid: DidUrl,
}
