use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use super::{Interface, Method};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ProtocolsConfigure {
    interface: Interface,
    method: Method,

    pub protocol_version: String,
    pub definition: ProtocolDefinition,
}

impl ProtocolsConfigure {
    pub fn new(definition: ProtocolDefinition) -> Self {
        ProtocolsConfigure {
            interface: Interface::Protocols,
            method: Method::Configure,

            protocol_version: "0.0.0".to_string(),
            definition,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ProtocolDefinition {
    pub protocol: String,
    pub published: bool,
    pub types: HashMap<String, ProtocolType>,
    pub structure: HashMap<String, ProtocolStructure>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ProtocolType {
    pub data_format: Vec<String>,
    pub schema: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtocolStructure {
    pub actions: Vec<Action>,
    pub children: HashMap<String, ProtocolStructure>,
}

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
                if key == "$actions" {
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
                "$actions".to_string(),
                serde_json::to_value(&self.actions).unwrap(),
            );
        }

        for (key, value) in &self.children {
            map.insert(key.to_string(), serde_json::to_value(value).unwrap());
        }

        map.serialize(serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Action {
    pub who: ActionWho,
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
