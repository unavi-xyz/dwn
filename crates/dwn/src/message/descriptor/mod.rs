use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use tracing::warn;

use self::{
    protocols::{ProtocolsConfigure, ProtocolsQuery},
    records::{RecordsDelete, RecordsQuery, RecordsRead, RecordsWrite},
};

pub mod protocols;
pub mod records;

pub use iana_media_types;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Interface {
    Permissions,
    Protocols,
    Records,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Method {
    Configure,
    Delete,
    Grant,
    Query,
    Read,
    Request,
    Revoke,
    Write,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Descriptor {
    PermissionsGrant,
    PermissionsQuery,
    PermissionsRequest,
    PermissionsRevoke,
    ProtocolsConfigure(ProtocolsConfigure),
    ProtocolsQuery(ProtocolsQuery),
    RecordsDelete(RecordsDelete),
    RecordsQuery(RecordsQuery),
    RecordsRead(RecordsRead),
    RecordsWrite(RecordsWrite),
}

impl From<ProtocolsConfigure> for Descriptor {
    fn from(desc: ProtocolsConfigure) -> Self {
        Descriptor::ProtocolsConfigure(desc)
    }
}

impl From<ProtocolsQuery> for Descriptor {
    fn from(desc: ProtocolsQuery) -> Self {
        Descriptor::ProtocolsQuery(desc)
    }
}

impl From<RecordsRead> for Descriptor {
    fn from(desc: RecordsRead) -> Self {
        Descriptor::RecordsRead(desc)
    }
}

impl From<RecordsQuery> for Descriptor {
    fn from(desc: RecordsQuery) -> Self {
        Descriptor::RecordsQuery(desc)
    }
}

impl From<RecordsWrite> for Descriptor {
    fn from(desc: RecordsWrite) -> Self {
        Descriptor::RecordsWrite(desc)
    }
}

impl From<RecordsDelete> for Descriptor {
    fn from(desc: RecordsDelete) -> Self {
        Descriptor::RecordsDelete(desc)
    }
}

impl<'de> Deserialize<'de> for Descriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Value::deserialize(deserializer)?;

        let interface = match json.get("interface").and_then(|i| i.as_str()) {
            Some(i) => i,
            None => {
                return Err(serde::de::Error::custom("Missing interface"));
            }
        };

        let method = match json.get("method").and_then(|m| m.as_str()) {
            Some(m) => m,
            None => {
                return Err(serde::de::Error::custom("Missing method"));
            }
        };

        match (interface, method) {
            ("Protocols", "Configure") => Ok(Descriptor::ProtocolsConfigure(
                serde_json::from_value(json).map_err(serde::de::Error::custom)?,
            )),
            ("Protocols", "Query") => Ok(Descriptor::ProtocolsQuery(
                serde_json::from_value(json).map_err(serde::de::Error::custom)?,
            )),
            ("Records", "Read") => Ok(Descriptor::RecordsRead(
                serde_json::from_value(json).map_err(serde::de::Error::custom)?,
            )),
            ("Records", "Query") => Ok(Descriptor::RecordsQuery(
                serde_json::from_value(json).map_err(serde::de::Error::custom)?,
            )),
            ("Records", "Write") => Ok(Descriptor::RecordsWrite(
                serde_json::from_value(json).map_err(serde::de::Error::custom)?,
            )),
            ("Records", "Delete") => Ok(Descriptor::RecordsDelete(
                serde_json::from_value(json).map_err(serde::de::Error::custom)?,
            )),
            _ => {
                warn!("Unsupported interface: {} {}", interface, method);
                Err(serde::de::Error::custom("Unsupported interface"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::encode::encode_cbor;

    use super::*;

    fn default_descriptors() -> Vec<Descriptor> {
        vec![
            Descriptor::ProtocolsConfigure(ProtocolsConfigure::default()),
            Descriptor::RecordsDelete(RecordsDelete::default()),
            Descriptor::RecordsQuery(RecordsQuery::default()),
            Descriptor::RecordsRead(RecordsRead::default()),
            Descriptor::RecordsWrite(RecordsWrite::default()),
        ]
    }

    #[test]
    fn test_serde() {
        for desc in default_descriptors() {
            let json = serde_json::to_value(&desc).unwrap();

            let desc2: Descriptor = serde_json::from_value(json).unwrap();
            assert_eq!(desc, desc2);
        }
    }

    #[test]
    fn test_encode_cbor() {
        for desc in default_descriptors() {
            encode_cbor(&desc).expect("Failed to generate CBOR block");
        }
    }
}
