use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::records;

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
        let interface = json.get("interface").expect("interface").as_str().unwrap();
        let method = json.get("method").expect("method").as_str().unwrap();

        match (interface, method) {
            ("Records", "Read") => Ok(Descriptor::RecordsRead(
                serde_json::from_value(json).unwrap(),
            )),
            ("Records", "Query") => Ok(Descriptor::RecordsQuery(
                serde_json::from_value(json).unwrap(),
            )),
            ("Records", "Write") => Ok(Descriptor::RecordsWrite(
                serde_json::from_value(json).unwrap(),
            )),
            ("Records", "Commit") => Ok(Descriptor::RecordsCommit(
                serde_json::from_value(json).unwrap(),
            )),
            ("Records", "Delete") => Ok(Descriptor::RecordsDelete(
                serde_json::from_value(json).unwrap(),
            )),
            _ => panic!("Unsupported interface and method combination"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::request::message::Message;

    use super::*;

    #[test]
    fn message_serialization() {
        let messages = vec![
            Message::new(records::RecordsRead::default()),
            Message::new(records::RecordsQuery::default()),
            Message::new(records::RecordsWrite::default()),
            Message::new(records::RecordsCommit::default()),
            Message::new(records::RecordsDelete::default()),
        ];

        for message in messages {
            let serialized = serde_json::to_string(&message).unwrap();
            let deserialized: Message = serde_json::from_str(&serialized).unwrap();
            assert_eq!(message, deserialized);
        }
    }
}
