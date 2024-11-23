use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::message::{
    cid::CidGenerationError,
    descriptor::{Descriptor, Interface, Method},
    Message,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordsRead {
    interface: Interface,
    method: Method,
    #[serde(with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    pub record_id: String,
}

pub struct RecordsReadBuilder {
    record_id: String,
}

impl RecordsReadBuilder {
    pub fn new(record_id: String) -> Self {
        Self { record_id }
    }

    pub fn build(self) -> Result<Message, CidGenerationError> {
        let descriptor = Descriptor::RecordsRead(Box::new(RecordsRead {
            interface: Interface::Records,
            method: Method::Read,
            record_id: self.record_id,
            message_timestamp: OffsetDateTime::now_utc(),
        }));

        Ok(Message {
            record_id: descriptor.compute_entry_id()?,
            data: None,
            descriptor,
            attestation: None,
            authorization: None,
        })
    }
}
