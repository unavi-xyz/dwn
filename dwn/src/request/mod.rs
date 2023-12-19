use crate::util::cid_from_bytes;
use data::Data;
use libipld_cbor::DagCborCodec;
use media_types::MediaType;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub mod data;

pub use iana_media_types as media_types;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestBody {
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    #[serde(rename = "recordId")]
    pub record_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    pub descriptor: Descriptor,
}

impl Message {
    pub fn new(data: Option<String>, descriptor: Descriptor) -> Self {
        let mut msg = Message {
            data,
            descriptor,
            record_id: "".to_string(),
        };

        msg.record_id = msg.generate_record_id().unwrap();

        msg
    }

    pub fn generate_record_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        let generator = RecordIdGenerator::try_from(&self.descriptor)?;
        let record_id = generator.generate_cid()?;
        Ok(record_id)
    }
}

pub struct MessageBuilder<T: Data> {
    pub data: Option<T>,
    pub descriptor: DescriptorBuilder,
}

impl<T: Data> MessageBuilder<T> {
    pub fn new(interface: Interface, method: Method, data: T) -> MessageBuilder<T> {
        MessageBuilder {
            data: Some(data),
            descriptor: DescriptorBuilder { interface, method },
        }
    }

    pub fn build(&self) -> Result<Message, Box<dyn std::error::Error>> {
        let data = self.data.as_ref().map(|d| d.to_base64url());
        let descriptor = self.descriptor.build(self.data.as_ref())?;
        Ok(Message::new(data, descriptor))
    }
}

#[derive(Serialize)]
pub struct RecordIdGenerator {
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
}

impl RecordIdGenerator {
    /// Generates the CID of this struct after DAG-CBOR serialization.
    pub fn generate_cid(&self) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = serde_ipld_dagcbor::to_vec(self)?;
        let cid = cid_from_bytes(DagCborCodec.into(), &bytes);
        Ok(cid.to_string())
    }
}

impl TryFrom<&Descriptor> for RecordIdGenerator {
    type Error = Box<dyn std::error::Error>;

    fn try_from(descriptor: &Descriptor) -> Result<Self, Self::Error> {
        let serialized = serde_ipld_dagcbor::to_vec(descriptor)?;
        let descriptor_cid = cid_from_bytes(DagCborCodec.into(), &serialized).to_string();
        Ok(RecordIdGenerator { descriptor_cid })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Descriptor {
    pub interface: Interface,
    pub method: Method,
    #[serde(rename = "dataCid", skip_serializing_if = "Option::is_none")]
    pub data_cid: Option<String>,
    #[serde(rename = "dataFormat", skip_serializing_if = "Option::is_none")]
    pub data_format: Option<MediaType>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Interface {
    /// For feature detection, the spec only lists a method, no interface.
    /// But the spec also says that an interface MUST exist in the descriptor.
    /// So we use this interface for feature detection, even though it doesn't exist in the spec.
    /// https://identity.foundation/decentralized-web-node/spec/#read
    FeatureDetection,
    Records,
    Protocols,
    Permissions,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Method {
    Commit,
    Configure,
    Delete,
    FeatureDetectionRead,
    Grant,
    Query,
    Read,
    Request,
    Revoke,
    Write,
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}

pub struct DescriptorBuilder {
    pub interface: Interface,
    pub method: Method,
}

impl DescriptorBuilder {
    pub fn build<T: Data>(
        &self,
        data: Option<&T>,
    ) -> Result<Descriptor, Box<dyn std::error::Error>> {
        let data_cid = data.map(|d| d.data_cid());
        let data_format = data.map(|d| d.data_format());

        Ok(Descriptor {
            interface: self.interface.clone(),
            method: self.method.clone(),
            data_cid,
            data_format,
        })
    }
}
