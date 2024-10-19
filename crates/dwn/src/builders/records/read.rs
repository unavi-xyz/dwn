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
            data_cid: None,
            data_format: None,
            parent_id: None,
            protocol: None,
            protocol_version: None,
            published: None,
            schema: None,
            date_created: OffsetDateTime::now_utc(),
            date_published: None,
        };

        Ok(Message {
            record_id: self.record_id,
            context_id: None,
            data: None,
            descriptor,
        })
    }
}
