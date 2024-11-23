use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::message::descriptor::{Interface, Method};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordsSync {
    interface: Interface,
    method: Method,
    pub local_records: Vec<RecordId>,
    #[serde(with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordId {
    pub record_id: String,
    pub latest_entry_id: String,
}

impl RecordsSync {
    pub fn new(local_records: Vec<RecordId>) -> Self {
        Self {
            interface: Interface::Records,
            method: Method::Sync,
            local_records,
            message_timestamp: OffsetDateTime::now_utc(),
        }
    }
}
