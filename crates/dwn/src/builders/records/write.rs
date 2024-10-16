use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use dwn_core::message::{
    cid::CidGenerationError,
    data::{compute_data_cid, Data},
    mime::Mime,
    Descriptor, Interface, Message, Method,
};

#[derive(Default)]
pub struct RecordsWriteBuilder {
    record_id: Option<String>,
    data: Option<Vec<u8>>,
    data_format: Option<Mime>,
}

impl RecordsWriteBuilder {
    pub fn record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }

    pub fn data(mut self, format: Mime, data: Vec<u8>) -> Self {
        self.data_format = Some(format);
        self.data = Some(data);
        self
    }

    pub fn build(self) -> Result<Message, CidGenerationError> {
        let data_cid = self.data.as_ref().and_then(|d| compute_data_cid(d));

        let descriptor = Descriptor {
            interface: Interface::Records,
            method: Method::Write,
            data_cid,
            data_format: self.data_format,
        };

        let record_id = match self.record_id {
            Some(v) => v,
            None => descriptor.compute_record_id()?,
        };

        let data = self
            .data
            .as_ref()
            .map(|d| Data::Base64(BASE64_URL_SAFE_NO_PAD.encode(d)));

        Ok(Message {
            record_id,
            data,
            descriptor,
        })
    }
}

#[cfg(test)]
mod tests {
    use dwn_core::message::mime::TEXT_PLAIN;

    use super::*;

    #[test]
    fn test_record_id_generation() {
        let msg = RecordsWriteBuilder::default().build().unwrap();
        assert_eq!(msg.record_id, msg.descriptor.compute_record_id().unwrap());
    }

    #[test]
    fn test_data() {
        let data = vec![0, 1, 2, 3, 2, 1, 0];
        let msg = RecordsWriteBuilder::default()
            .data(TEXT_PLAIN, data.clone())
            .build()
            .unwrap();

        let encoded = match msg.data.as_ref().unwrap() {
            Data::Base64(s) => s,
            Data::Encrypted(_) => panic!("data encrypted"),
        };
        let parsed = BASE64_URL_SAFE_NO_PAD.decode(encoded).unwrap();
        assert_eq!(parsed, data);
    }
}
