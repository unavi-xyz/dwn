use std::collections::BTreeMap;

use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use self::descriptor::Descriptor;

pub mod descriptor;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Message {
    pub attestation: Option<JWS>,
    pub authorization: Option<JWS>,
    pub descriptor: Descriptor,
    #[serde(rename = "recordId")]
    pub record_id: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct JWS {
    pub payload: Option<String>,
    pub signatures: Option<Vec<SignatureEntry>>,
    pub header: Option<BTreeMap<String, Ipld>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SignatureEntry {
    pub payload: Option<String>,
    pub protected: Option<String>,
    pub signature: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}
