pub use iana_media_types as media_types;
// use libipld_core::{codec::Codec, ipld::Ipld};
// use libipld_pb::DagPbCodec;
use serde::{Deserialize, Serialize};

use crate::util::cid_from_bytes;

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

pub struct MessageBuilder {
    pub data: Option<String>,
    pub descriptor: DescriptorBuilder,
}

impl MessageBuilder {
    pub fn build(&self) -> Result<Message, Box<dyn std::error::Error>> {
        Ok(Message {
            record_id: self.generate_record_id()?,
            data: self.data.clone(), // TODO: bas64 encode data
            descriptor: self.descriptor.build()?,
        })
    }

    pub fn generate_record_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        let descriptor = self.descriptor.build()?;
        let generator = RecordIdGenerator::try_from(&descriptor)?;
        let record_id = generator.generate_cid()?;
        Ok(record_id)
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
        let cid = cid_from_bytes(&bytes);
        Ok(cid.to_string())
    }
}

impl TryFrom<&Descriptor> for RecordIdGenerator {
    type Error = Box<dyn std::error::Error>;

    fn try_from(descriptor: &Descriptor) -> Result<Self, Self::Error> {
        let serialized = serde_ipld_dagcbor::to_vec(descriptor)?;
        let descriptor_cid = cid_from_bytes(&serialized).to_string();
        Ok(RecordIdGenerator { descriptor_cid })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Descriptor {
    pub interface: String,
    pub method: String,
    #[serde(rename = "dataCid", skip_serializing_if = "Option::is_none")]
    pub data_cid: Option<String>,
    #[serde(rename = "dataFormat", skip_serializing_if = "Option::is_none")]
    pub data_format: Option<String>,
}

pub struct DescriptorBuilder {
    pub interface: String,
    pub method: String,
    pub data_format: Option<media_types::MediaType>,
}

impl DescriptorBuilder {
    pub fn build(&self) -> Result<Descriptor, Box<dyn std::error::Error>> {
        let data_format = self.data_format.as_ref().map(|f| f.to_string());

        Ok(Descriptor {
            interface: self.interface.clone(),
            method: self.method.clone(),
            data_cid: None, // TODO: Generate data_cid
            data_format,
        })
    }
}
