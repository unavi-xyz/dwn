use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};

use crate::message::descriptor::{Interface, Method};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolsConfigure {
    interface: Interface,
    method: Method,
    pub protocol_version: semver::Version,
    pub definition: Option<ProtocolDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolDefinition {
    pub protocol: String,
    pub published: bool,
    pub types: HashMap<String, ProtocolType>,
    pub structure: HashMap<String, ProtocolStructure>,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolType {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub data_formats: Vec<mime::Mime>,
    pub schema: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolStructure {
    #[serde(rename = "$actions")]
    pub actions: Option<Vec<ProtocolRule>>,
    #[serde(flatten)]
    pub children: HashMap<String, ProtocolStructure>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolRule {
    pub who: Who,
    pub can: Can,
    pub of: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Who {
    Anyone,
    Author,
    Recipient,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Can {
    Read,
    Write,
}
