use std::collections::BTreeMap;

use libipld::{Block, DefaultParams};
use libipld_cbor::DagCborCodec;
use libipld_core::{error::SerdeError, ipld::Ipld, multihash::Code, serde::to_ipld};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use self::descriptor::Descriptor;

pub mod descriptor;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Message {
    pub attestation: Option<JWS>,
    pub authorization: Option<JWS>,
    pub data: Option<Data>,
    pub descriptor: Descriptor,
    #[serde(rename = "recordId")]
    pub record_id: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum CidError {
    #[error("Failed to serialize descriptor to IPLD: {0}")]
    Serde(#[from] SerdeError),
    #[error("Failed to encode descriptor to CBOR: {0}")]
    Encode(#[from] anyhow::Error),
}

impl Message {
    /// Generate a CBOR encoded IPLD block from the message
    pub fn cbor_block(&self) -> Result<Block<DefaultParams>, CidError> {
        let ipld = to_ipld(self)?;
        let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &ipld)?;
        Ok(block)
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct JWS {
    pub header: Option<BTreeMap<String, Ipld>>,
    pub payload: Option<String>,
    pub signatures: Option<Vec<SignatureEntry>>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SignatureEntry {
    pub payload: Option<String>,
    pub protected: Option<String>,
    pub signature: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Data {
    Base64(String),
    Encrypted(EncryptedData),
}

impl Data {
    pub fn encode(&self) -> Result<Ipld, SerdeError> {
        to_ipld(self)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EncryptedData {
    pub protected: String,
    pub recipients: Vec<String>,
    pub ciphertext: String,
    pub iv: String,
    pub tag: String,
}
