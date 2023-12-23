use libipld_cbor::DagCborCodec;
use serde::{Deserialize, Serialize};

use crate::util::cid_from_bytes;

use super::descriptor::Descriptor;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Message {
    #[serde(rename = "recordId")]
    pub record_id: String,
    pub descriptor: Descriptor,
}

impl Message {
    pub fn new<T: Serialize + Into<Descriptor>>(descriptor: T) -> Self {
        let mut msg = Message {
            record_id: "".to_string(),
            descriptor: descriptor.into(),
        };

        msg.record_id = msg.generate_record_id().unwrap();

        msg
    }

    /// Returns the generated record ID for the message.
    pub fn generate_record_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        let generator = RecordIdGenerator::new(&self.descriptor)?;
        generator.generate()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum CommitStrategy {
    #[serde(rename = "json-patch")]
    JsonPatch,
    #[serde(rename = "json-merge")]
    JsonMerge,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Encryption {
    #[serde(rename = "AES-GCM")]
    AesGcm,
    #[serde(rename = "XSalsa20-Poly1305")]
    XSalsa20Poly1305,
}

#[derive(Serialize)]
struct RecordIdGenerator {
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
}

impl RecordIdGenerator {
    pub fn new<T: Serialize>(descriptor: &T) -> Result<Self, Box<dyn std::error::Error>> {
        let serialized = serde_ipld_dagcbor::to_vec(descriptor)?;
        let descriptor_cid = cid_from_bytes(DagCborCodec.into(), &serialized).to_string();
        Ok(RecordIdGenerator { descriptor_cid })
    }

    pub fn generate(&self) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = serde_ipld_dagcbor::to_vec(self)?;
        let cid = cid_from_bytes(DagCborCodec.into(), &bytes);
        Ok(cid.to_string())
    }
}
