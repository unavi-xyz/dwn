use std::collections::HashMap;

use iana_media_types::MediaType;
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize};

use crate::message::descriptor::{Interface, Method};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ProtocolsConfigure {
    interface: Interface,
    method: Method,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<ProtocolDefinition>,
    #[serde(rename = "lastConfiguration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_configuration: Option<String>,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: Version,
}

impl Default for ProtocolsConfigure {
    fn default() -> Self {
        ProtocolsConfigure {
            interface: Interface::Protocols,
            method: Method::Configure,

            definition: None,
            last_configuration: None,
            protocol_version: Version::new(0, 0, 0),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ProtocolDefinition {
    pub protocol: String,
    pub published: bool,
    pub types: HashMap<String, StructureType>,
    pub structure: HashMap<String, ProtocolStructure>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct StructureType {
    #[serde(rename = "dataFormat")]
    pub data_format: Vec<MediaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProtocolStructure {
    pub actions: Vec<Action>,
    pub children: HashMap<String, ProtocolStructure>,
}

const ACTIONS_KEY: &str = "$actions";

impl<'de> Deserialize<'de> for ProtocolStructure {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut children = HashMap::new();
        let mut actions = Vec::new();

        let value = serde_json::Value::deserialize(deserializer)?;

        if let serde_json::Value::Object(map) = value {
            for (key, value) in map {
                if key == ACTIONS_KEY {
                    actions = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                } else {
                    children.insert(
                        key,
                        serde_json::from_value(value).map_err(serde::de::Error::custom)?,
                    );
                }
            }
        }

        Ok(ProtocolStructure { actions, children })
    }
}

impl Serialize for ProtocolStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut map = serde_json::Map::new();

        if !self.actions.is_empty() {
            map.insert(
                ACTIONS_KEY.to_string(),
                serde_json::to_value(&self.actions).map_err(serde::ser::Error::custom)?,
            );
        }

        for (key, value) in &self.children {
            map.insert(
                key.to_string(),
                serde_json::to_value(value).map_err(serde::ser::Error::custom)?,
            );
        }

        map.serialize(serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Action {
    pub who: ActionWho,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub of: Option<String>,
    pub can: ActionCan,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
pub enum ActionWho {
    #[serde(rename = "anyone")]
    Anyone,
    #[serde(rename = "author")]
    Author,
    #[serde(rename = "recipient")]
    Recipient,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
pub enum ActionCan {
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
}
