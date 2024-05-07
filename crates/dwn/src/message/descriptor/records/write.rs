use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::message::descriptor::{Interface, Method};

pub use semver::Version;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsWrite {
    interface: Interface,
    method: Method,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_cid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_format: Option<String>,
    #[serde(rename = "datePublished", with = "time::serde::rfc3339::option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_published: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<Encryption>,
    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    #[serde(rename = "parentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(rename = "protocolPath")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_path: Option<String>,
    #[serde(rename = "protocolVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<Version>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
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

impl Default for RecordsWrite {
    fn default() -> Self {
        let time = OffsetDateTime::now_utc();

        RecordsWrite {
            interface: Interface::Records,
            method: Method::Write,

            data_cid: None,
            data_format: None,
            date_published: Some(time),
            encryption: None,
            message_timestamp: time,
            parent_id: None,
            protocol: None,
            protocol_path: None,
            protocol_version: None,
            published: None,
            schema: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_serde() {
        let version = Version::new(1, 0, 0);
        let json = serde_json::to_string(&version).unwrap();
        assert_eq!(json, r#""1.0.0""#);

        let write = RecordsWrite {
            protocol_version: Some(Version::new(1, 0, 0)),
            ..Default::default()
        };

        let json = serde_json::to_string(&write).unwrap();
        let write: RecordsWrite = serde_json::from_str(&json).unwrap();

        assert_eq!(write.protocol_version, Some(Version::new(1, 0, 0)));
    }
}
