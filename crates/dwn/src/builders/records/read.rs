use dwn_core::message::{
    cid::CidGenerationError, Descriptor, Interface, Message, Method, OffsetDateTime,
};

pub struct RecordsReadBuilder {
    record_id: String,
}

impl RecordsReadBuilder {
    pub fn new(record_id: String) -> Self {
        Self { record_id }
    }

    pub fn build(self) -> Result<Message, CidGenerationError> {
        let descriptor = Descriptor {
            interface: Interface::Records,
            method: Method::Read,
            filter: None,
            data_cid: None,
            data_format: None,
            protocol: None,
            protocol_version: None,
            parent_id: None,
            published: None,
            schema: None,
            message_timestamp: OffsetDateTime::now_utc(),
        };

        Ok(Message {
            record_id: self.record_id,
            data: None,
            descriptor,
            attestation: None,
            authorization: None,
        })
    }
}
