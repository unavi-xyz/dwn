use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use time::OffsetDateTime;

use crate::message::descriptor::{Interface, Method};

pub use semver::Version;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsWrite {
    interface: Interface,
    method: Method,

    #[serde(rename = "contextId")]
    pub context_id: Option<String>,
    pub data_cid: Option<String>,
    #[serde(rename = "datePublished", with = "time::serde::rfc3339::option")]
    pub date_published: Option<OffsetDateTime>,
    pub encryption: Option<Encryption>,
    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub protocol: Option<String>,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: Option<Version>,
    pub published: Option<bool>,
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

            context_id: None,
            data_cid: None,
            date_published: Some(time),
            encryption: None,
            message_timestamp: time,
            parent_id: None,
            protocol: None,
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
