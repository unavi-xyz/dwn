use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::message::{
    Message,
    cid::CidGenerationError,
    descriptor::{Descriptor, Interface, Method},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordsDelete {
    interface: Interface,
    method: Method,
    #[serde(with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    pub record_id: String,
}

pub struct RecordsDeleteBuilder {
    record_id: String,
}

impl RecordsDeleteBuilder {
    pub fn new(record_id: String) -> Self {
        Self { record_id }
    }

    pub fn build(self) -> Result<Message, CidGenerationError> {
        let descriptor = Descriptor::RecordsDelete(Box::new(RecordsDelete {
            interface: Interface::Records,
            method: Method::Delete,
            record_id: self.record_id,
            message_timestamp: OffsetDateTime::now_utc(),
        }));

        Ok(Message {
            record_id: descriptor.compute_entry_id()?,
            context_id: None,
            data: None,
            descriptor,
            attestation: None,
            authorization: None,
        })
    }
}
