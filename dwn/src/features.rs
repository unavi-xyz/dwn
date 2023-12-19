use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FeatureDetection {
    #[serde(rename = "type")]
    pub type_: String,
    pub interfaces: Interfaces,
}

impl Default for FeatureDetection {
    fn default() -> Self {
        Self {
            type_: "FeatureDetection".to_string(),
            interfaces: Interfaces {
                protocols: None,
                records: None,
                permissions: None,
                messaging: None,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Interfaces {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<BTreeMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub records: Option<BTreeMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<BTreeMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messaging: Option<Messaging>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Messaging {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batching: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Protocol {
    ProtocolsConfigure,
    ProtocolsQuery,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Records {
    RecordsCommit,
    RecordsDelete,
    RecordsQuery,
    RecordsWrite,
}

impl Display for Records {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Permission {
    PermissionsGrant,
    PermissionsRevoke,
}

impl Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}
