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
    pub data: Option<String>,
    pub descriptor: Descriptor,
}

impl Message {
    pub fn new(
        data: Option<String>,
        descriptor: Descriptor,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut message = Message {
            record_id: String::new(),
            data,
            descriptor,
        };

        message.generate_record_id()?;

        Ok(message)
    }

    pub fn generate_record_id(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let generator = RecordIdGenerator::try_from(&self.descriptor)?;
        self.record_id = generator.generate_cid()?;
        Ok(())
    }

    pub fn generate_data_cid(&mut self) {
        self.descriptor.data_cid = match &self.data {
            Some(_) => {
                todo!();
                // let pb = Ipld::from();
                // let bytes = DagPbCodec.encode(&pb).unwrap();
                // Some(cid_from_bytes(&bytes).to_string())
            }
            None => None,
        };
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
    #[serde(rename = "dataCid")]
    pub data_cid: Option<String>,
    #[serde(rename = "dataFormat")]
    pub data_format: Option<String>,
}
