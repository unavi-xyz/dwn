use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};
use serde::{Deserialize, Serialize};

const RAW: u64 = 0x55;

fn cid_from_bytes(bytes: &[u8]) -> Cid {
    let hash = Code::Sha2_256.digest(bytes);
    Cid::new_v1(RAW, hash)
}

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
    pub fn generate_record_id(&mut self) {
        let generator = CidGenerator::from(&self.descriptor);

        let bytes = match serde_ipld_dagcbor::to_vec(&generator) {
            Ok(bytes) => bytes,
            Err(err) => panic!("Failed to serialize generator: {}", err),
        };

        self.record_id = cid_from_bytes(&bytes).to_string();
    }
}

#[derive(Serialize)]
pub struct CidGenerator {
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
}

impl From<&Descriptor> for CidGenerator {
    fn from(descriptor: &Descriptor) -> Self {
        let serialized = match serde_ipld_dagcbor::to_vec(descriptor) {
            Ok(bytes) => bytes,
            Err(err) => panic!("Failed to serialize descriptor: {}", err),
        };

        let descriptor_cid = cid_from_bytes(&serialized).to_string();

        CidGenerator { descriptor_cid }
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
