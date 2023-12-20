use serde::{Deserialize, Deserializer, Serialize};

pub mod data;
pub mod message;
pub mod records;

pub use iana_media_types as media_types;
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestBody {
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Message {
    RecordsRead(records::RecordsRead),
    RecordsQuery(records::RecordsQuery),
    RecordsWrite(records::RecordsWrite),
    RecordsCommit(records::RecordsCommit),
    RecordsDelete(records::RecordsDelete),
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Value::deserialize(deserializer)?;
        let descriptor = json
            .get("descriptor")
            .expect("descriptor")
            .as_object()
            .unwrap();

        let interface = descriptor
            .get("interface")
            .expect("interface")
            .as_str()
            .unwrap();
        let method = descriptor.get("method").expect("method").as_str().unwrap();

        match (interface, method) {
            ("Records", "Read") => Ok(Message::RecordsRead(serde_json::from_value(json).unwrap())),
            ("Records", "Query") => {
                Ok(Message::RecordsQuery(serde_json::from_value(json).unwrap()))
            }
            ("Records", "Write") => {
                Ok(Message::RecordsWrite(serde_json::from_value(json).unwrap()))
            }
            ("Records", "Commit") => Ok(Message::RecordsCommit(
                serde_json::from_value(json).unwrap(),
            )),
            ("Records", "Delete") => Ok(Message::RecordsDelete(
                serde_json::from_value(json).unwrap(),
            )),
            _ => panic!("Unsupported interface and method combination"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_serialization() {
        let messages = vec![
            Message::RecordsRead(records::RecordsRead::default()),
            Message::RecordsQuery(records::RecordsQuery::default()),
            Message::RecordsWrite(records::RecordsWrite::default()),
            Message::RecordsCommit(records::RecordsCommit::default()),
            Message::RecordsDelete(records::RecordsDelete::default()),
        ];

        for message in messages {
            let serialized = serde_json::to_string(&message).unwrap();
            let deserialized: Message = serde_json::from_str(&serialized).unwrap();
            assert_eq!(message, deserialized);
        }
    }
}
