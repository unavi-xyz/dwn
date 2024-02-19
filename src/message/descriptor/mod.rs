use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use tracing::warn;

mod records;

pub use records::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Interface {
    Permissions,
    Protocols,
    Records,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Method {
    Commit,
    Configure,
    Delete,
    Grant,
    Query,
    Read,
    Request,
    Revoke,
    Write,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Encryption {
    /// AES-GCM
    #[serde(rename = "jwe")]
    JWE,
    /// XSalsa20-Poly1305
    #[serde(rename = "X25519")]
    X25519,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Descriptor {
    RecordsRead(records::RecordsRead),
    RecordsQuery(records::RecordsQuery),
    RecordsWrite(records::RecordsWrite),
    RecordsCommit(records::RecordsCommit),
    RecordsDelete(records::RecordsDelete),
}

impl From<records::RecordsRead> for Descriptor {
    fn from(desc: records::RecordsRead) -> Self {
        Descriptor::RecordsRead(desc)
    }
}

impl From<records::RecordsQuery> for Descriptor {
    fn from(desc: records::RecordsQuery) -> Self {
        Descriptor::RecordsQuery(desc)
    }
}

impl From<records::RecordsWrite> for Descriptor {
    fn from(desc: records::RecordsWrite) -> Self {
        Descriptor::RecordsWrite(desc)
    }
}

impl From<records::RecordsCommit> for Descriptor {
    fn from(desc: records::RecordsCommit) -> Self {
        Descriptor::RecordsCommit(desc)
    }
}

impl From<records::RecordsDelete> for Descriptor {
    fn from(desc: records::RecordsDelete) -> Self {
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
                warn!("Missing interface");
                return Err(serde::de::Error::custom("Missing interface"));
            }
        };

        let method = match json.get("method").and_then(|m| m.as_str()) {
            Some(m) => m,
            None => {
                warn!("Missing method");
                return Err(serde::de::Error::custom("Missing method"));
            }
        };

        match (interface, method) {
            ("Records", "Read") => Ok(Descriptor::RecordsRead(
                serde_json::from_value(json).unwrap_or_else(|e| {
                    warn!("Failed to deserialize RecordsRead: {}", e);
                    records::RecordsRead::default()
                }),
            )),
            ("Records", "Query") => Ok(Descriptor::RecordsQuery(
                serde_json::from_value(json).unwrap_or_else(|e| {
                    warn!("Failed to deserialize RecordsQuery: {}", e);
                    records::RecordsQuery::default()
                }),
            )),
            ("Records", "Write") => Ok(Descriptor::RecordsWrite(
                serde_json::from_value(json).unwrap_or_else(|e| {
                    warn!("Failed to deserialize RecordsWrite: {}", e);
                    records::RecordsWrite::default()
                }),
            )),
            ("Records", "Commit") => Ok(Descriptor::RecordsCommit(
                serde_json::from_value(json).unwrap_or_else(|e| {
                    warn!("Failed to deserialize RecordsCommit: {}", e);
                    records::RecordsCommit::default()
                }),
            )),
            ("Records", "Delete") => Ok(Descriptor::RecordsDelete(
                serde_json::from_value(json).unwrap_or_else(|e| {
                    warn!("Failed to deserialize RecordsDelete: {}", e);
                    records::RecordsDelete::default()
                }),
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
    use crate::util::encode_cbor;

    use super::*;

    fn default_descriptors() -> Vec<Descriptor> {
        vec![
            Descriptor::RecordsCommit(records::RecordsCommit::default()),
            Descriptor::RecordsDelete(records::RecordsDelete::default()),
            Descriptor::RecordsQuery(records::RecordsQuery::default()),
            Descriptor::RecordsRead(records::RecordsRead::default()),
            Descriptor::RecordsWrite(records::RecordsWrite::default()),
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
