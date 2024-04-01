use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::message::descriptor::{Interface, Method};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsDelete {
    interface: Interface,
    method: Method,
    #[serde(rename = "recordId")]
    pub record_id: String,
    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
}

impl RecordsDelete {
    pub fn new(record_id: String) -> Self {
        RecordsDelete {
            record_id,
            ..Default::default()
        }
    }
}

impl Default for RecordsDelete {
    fn default() -> Self {
        RecordsDelete {
            interface: Interface::Records,
            method: Method::Delete,
            record_id: "".to_string(),
            message_timestamp: OffsetDateTime::now_utc(),
        }
    }
}
