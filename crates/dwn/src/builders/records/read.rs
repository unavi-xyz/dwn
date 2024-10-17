use dwn_core::message::{cid::CidGenerationError, Descriptor, Interface, Message, Method};

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
        };

        Ok(Message {
            record_id: self.record_id,
            data: None,
            descriptor,
        })
    }
}
