use std::collections::HashMap;

use semver::Version;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as, skip_serializing_none};

use crate::message::{
    Message,
    cid::CidGenerationError,
    descriptor::{Descriptor, Interface, Method},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolsConfigure {
    interface: Interface,
    method: Method,
    pub protocol_version: semver::Version,
    pub definition: ProtocolDefinition,
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
    pub data_format: Vec<mime::Mime>,
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
#[serde(rename_all = "camelCase")]
pub enum Who {
    Anyone,
    Author,
    Recipient,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Can {
    Create,
    Delete,
    Query,
    Read,
    Subscribe,
    Update,
}

pub struct ProtocolsConfigureBuilder {
    version: Version,
    definition: ProtocolDefinition,
}

impl ProtocolsConfigureBuilder {
    pub fn new(version: Version, definition: ProtocolDefinition) -> Self {
        Self {
            version,
            definition,
        }
    }

    pub fn build(self) -> Result<Message, CidGenerationError> {
        let descriptor = Descriptor::ProtocolsConfigure(Box::new(ProtocolsConfigure {
            interface: Interface::Records,
            method: Method::Delete,
            protocol_version: self.version,
            definition: self.definition,
        }));

        Ok(Message {
            record_id: descriptor.compute_entry_id()?,
            context_id: None,
            data: None,
            descriptor,
            attestation: None,
            authorization: None,
        })
    }
}
