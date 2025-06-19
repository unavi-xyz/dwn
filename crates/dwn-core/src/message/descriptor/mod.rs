use std::fmt::Display;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

mod protocols;
mod records;

pub use protocols::*;
pub use records::*;
use time::OffsetDateTime;

use super::cid::{CidGenerationError, compute_cid_cbor};

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Descriptor {
    ProtocolsConfigure(Box<ProtocolsConfigure>),
    ProtocolsQuery(Box<ProtocolsQuery>),
    RecordsQuery(Box<RecordsQuery>),
    RecordsRead(Box<RecordsRead>),
    RecordsSync(Box<RecordsSync>),
    RecordsWrite(Box<RecordsWrite>),
}

impl Descriptor {
    pub fn compute_entry_id(&self) -> Result<String, CidGenerationError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct RecordIdGeneration {
            descriptor_cid: String,
        }

        let generator = RecordIdGeneration {
            descriptor_cid: compute_cid_cbor(self)?,
        };

        compute_cid_cbor(&generator)
    }

    pub fn message_timestamp(&self) -> Option<&OffsetDateTime> {
        match self {
            Descriptor::ProtocolsConfigure(_) => None,
            Descriptor::ProtocolsQuery(_) => None,
            Descriptor::RecordsQuery(d) => Some(&d.message_timestamp),
            Descriptor::RecordsRead(d) => Some(&d.message_timestamp),
            Descriptor::RecordsSync(d) => Some(&d.message_timestamp),
            Descriptor::RecordsWrite(d) => Some(&d.message_timestamp),
        }
    }
}

impl<'de> Deserialize<'de> for Descriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Value::deserialize(deserializer)?;

        let Some(interface) = raw.get("interface") else {
            return Err(serde::de::Error::custom("no interface"));
        };
        let interface = serde_json::from_value::<Interface>(interface.clone())
            .map_err(|_| serde::de::Error::custom("unsupported interface"))?;

        let Some(method) = raw.get("method") else {
            return Err(serde::de::Error::custom("no method"));
        };
        let method = serde_json::from_value::<Method>(method.clone())
            .map_err(|_| serde::de::Error::custom("unsupported method"))?;

        match (interface, method) {
            (Interface::Records, Method::Query) => {
                let desc: RecordsQuery =
                    serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
                Ok(Descriptor::RecordsQuery(Box::new(desc)))
            }
            (Interface::Records, Method::Read) => {
                let desc: RecordsRead =
                    serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
                Ok(Descriptor::RecordsRead(Box::new(desc)))
            }
            (Interface::Records, Method::Sync) => {
                let desc: RecordsSync =
                    serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
                Ok(Descriptor::RecordsSync(Box::new(desc)))
            }
            (Interface::Records, Method::Write) => {
                let desc: RecordsWrite =
                    serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
                Ok(Descriptor::RecordsWrite(Box::new(desc)))
            }
            _ => Err(serde::de::Error::custom(
                "Unsupported interface / method combination",
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Interface {
    Protocols,
    Records,
}

impl Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Query,
    Read,
    Sync,
    Write,
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use mime::TEXT_PLAIN;
    use semver::Version;

    use crate::message::Message;

    use super::*;

    #[test]
    fn test_serialize_records_query() {
        let msg = RecordsQueryBuilder::default()
            .schema("schema".to_string())
            .protocol("protocol".to_string(), Version::new(1, 2, 3))
            .record_id("record id".to_string())
            .parent_id("parent id".to_string())
            .build()
            .unwrap();
        let ser = serde_json::to_string_pretty(&msg).unwrap();
        println!("{}", ser);
        let des = serde_json::from_str::<Message>(&ser).unwrap();
        assert_eq!(des, msg);
    }

    #[test]
    fn test_serialize_records_read() {
        let msg = RecordsReadBuilder::new("test".to_string()).build().unwrap();
        let ser = serde_json::to_string_pretty(&msg).unwrap();
        println!("{}", ser);
        let des = serde_json::from_str::<Message>(&ser).unwrap();
        assert_eq!(des, msg);
    }

    #[test]
    fn test_serialize_records_write() {
        let msg = RecordsWriteBuilder::default()
            .data(TEXT_PLAIN, vec![0, 1, 2, 3])
            .schema("schema".to_string())
            .protocol("protocol".to_string(), Version::new(1, 2, 3))
            .record_id("record id".to_string())
            .parent_id("parent id".to_string())
            .published(true)
            .build()
            .unwrap();
        let ser = serde_json::to_string_pretty(&msg).unwrap();
        println!("{}", ser);
        let des = serde_json::from_str::<Message>(&ser).unwrap();
        assert_eq!(des, msg);
    }
}
