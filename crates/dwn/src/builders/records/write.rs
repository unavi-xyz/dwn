use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use dwn_core::message::{
    cid::CidGenerationError,
    data::{compute_data_cid, Data},
    mime::Mime,
    Descriptor, Interface, Message, Method, OffsetDateTime, Version,
};

#[derive(Default)]
pub struct RecordsWriteBuilder {
    record_id: Option<String>,
    context_id: Option<String>,
    data: Option<Vec<u8>>,
    data_format: Option<Mime>,
    schema: Option<String>,
    protocol: Option<String>,
    protocol_version: Option<Version>,
    parent_id: Option<String>,
    published: Option<bool>,
}

impl RecordsWriteBuilder {
    pub fn record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }

    pub fn context_id(mut self, value: String) -> Self {
        self.context_id = Some(value);
        self
    }

    pub fn data(mut self, format: Mime, data: Vec<u8>) -> Self {
        self.data_format = Some(format);
        self.data = Some(data);
        self
    }

    pub fn schema(mut self, value: String) -> Self {
        self.schema = Some(value);
        self
    }

    pub fn protocol(mut self, protocol: String, version: Version) -> Self {
        self.protocol = Some(protocol);
        self.protocol_version = Some(version);
        self
    }

    pub fn parent_id(mut self, value: String) -> Self {
        self.parent_id = Some(value);
        self
    }

    pub fn published(mut self, value: bool) -> Self {
        self.published = Some(value);
        self
    }

    pub fn build(self) -> Result<Message, CidGenerationError> {
        let data_cid = self.data.as_ref().and_then(|d| compute_data_cid(d));

        let descriptor = Descriptor {
            interface: Interface::Records,
            method: Method::Write,
            data_cid,
            data_format: self.data_format,
            schema: self.schema,
            protocol: self.protocol,
            protocol_version: self.protocol_version,
            parent_id: self.parent_id,
            published: self.published,
            date_created: OffsetDateTime::now_utc(),
            date_published: None,
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
            context_id: self.context_id,
            data,
            descriptor,
            attestation: None,
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
        let data = "test data".as_bytes().to_owned();

        let msg = RecordsWriteBuilder::default()
            .data(TEXT_PLAIN, data.clone())
            .build()
            .unwrap();
        assert!(msg.descriptor.data_cid.is_some());
        assert_eq!(msg.descriptor.data_format, Some(TEXT_PLAIN));

        let encoded = match msg.data.as_ref().unwrap() {
            Data::Base64(s) => s,
            Data::Encrypted(_) => panic!("data encrypted"),
        };
        let parsed = BASE64_URL_SAFE_NO_PAD.decode(encoded).unwrap();
        assert_eq!(parsed, data);
    }
}
