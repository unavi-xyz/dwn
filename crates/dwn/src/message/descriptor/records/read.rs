use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::message::descriptor::{Interface, Method};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsRead {
    interface: Interface,
    method: Method,

    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    #[serde(rename = "recordId")]
    pub record_id: String,
}

impl RecordsRead {
    pub fn new(record_id: String) -> Self {
        RecordsRead {
            record_id,
            ..Default::default()
        }
    }
}

impl Default for RecordsRead {
    fn default() -> Self {
        RecordsRead {
            interface: Interface::Records,
            method: Method::Read,
            message_timestamp: OffsetDateTime::now_utc(),
            record_id: "".to_string(),
        }
    }
}
