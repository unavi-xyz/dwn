use std::collections::BTreeMap;

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
