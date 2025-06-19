use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as, skip_serializing_none};
use time::OffsetDateTime;

use crate::message::{
    Message,
    cid::CidGenerationError,
    data::{Data, compute_data_cid},
    descriptor::{Descriptor, Interface, Method},
};

#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordsWrite {
    interface: Interface,
    method: Method,
    pub data_cid: Option<String>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub data_format: Option<mime::Mime>,
    #[serde(with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    pub parent_id: Option<String>,
    pub protocol: Option<String>,
    pub protocol_version: Option<semver::Version>,
    pub published: Option<bool>,
    pub schema: Option<String>,
}

#[derive(Default)]
pub struct RecordsWriteBuilder {
    record_id: Option<String>,
    data: Option<Vec<u8>>,
    data_format: Option<mime::Mime>,
    schema: Option<String>,
    protocol: Option<String>,
    protocol_version: Option<semver::Version>,
    parent_id: Option<String>,
    published: Option<bool>,
}

impl RecordsWriteBuilder {
    pub fn record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }

    pub fn data(mut self, format: mime::Mime, data: Vec<u8>) -> Self {
        self.data_format = Some(format);
        self.data = Some(data);
        self
    }

    pub fn schema(mut self, value: String) -> Self {
        self.schema = Some(value);
        self
    }

    pub fn protocol(mut self, protocol: String, version: semver::Version) -> Self {
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

        let descriptor = Descriptor::RecordsWrite(Box::new(RecordsWrite {
            interface: Interface::Records,
            method: Method::Write,
            data_cid,
            data_format: self.data_format,
            schema: self.schema,
            protocol: self.protocol,
            protocol_version: self.protocol_version,
            parent_id: self.parent_id,
            published: self.published,
            message_timestamp: OffsetDateTime::now_utc(),
        }));

        let record_id = match self.record_id {
            Some(v) => v,
            None => descriptor.compute_entry_id()?,
        };

        let data = self
            .data
            .as_ref()
            .map(|d| Data::Base64(BASE64_URL_SAFE_NO_PAD.encode(d)));

        Ok(Message {
            record_id,
            data,
            descriptor,
            attestation: None,
            authorization: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use mime::TEXT_PLAIN;

    use super::*;

    #[test]
    fn test_record_id_generation() {
        let msg = RecordsWriteBuilder::default().build().unwrap();
        assert_eq!(msg.record_id, msg.descriptor.compute_entry_id().unwrap());
    }

    #[test]
    fn test_data() {
        let data = "test data".as_bytes().to_owned();

        let msg = RecordsWriteBuilder::default()
            .data(TEXT_PLAIN, data.clone())
            .build()
            .unwrap();

        let Descriptor::RecordsWrite(desc) = msg.descriptor else {
            panic!()
        };
        assert!(desc.data_cid.is_some());
        assert_eq!(desc.data_format, Some(TEXT_PLAIN));

        let encoded = match msg.data.as_ref().unwrap() {
            Data::Base64(s) => s,
            Data::Encrypted(_) => panic!("data encrypted"),
        };
        let parsed = BASE64_URL_SAFE_NO_PAD.decode(encoded).unwrap();
        assert_eq!(parsed, data);
    }
}
