use std::collections::BTreeMap;

use libipld::{Block, DefaultParams};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    error::SerdeError,
    ipld::Ipld,
    multihash::Code,
    serde::{from_ipld, to_ipld},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("Failed to serialize to IPLD: {0}")]
    Serde(#[from] SerdeError),
    #[error("Failed to encode to CBOR: {0}")]
    Encode(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Failed to serialize to IPLD: {0}")]
    Serde(#[from] SerdeError),
    #[error("Failed to decode CBOR: {0}")]
    Decode(#[from] anyhow::Error),
}

impl Message {
    /// Encodes the message to a CBOR block
    pub fn encode_block(&self) -> Result<Block<DefaultParams>, EncodeError> {
        let ipld = to_ipld(self)?;
        let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &ipld)?;
        Ok(block)
    }

    /// Decodes a CBOR block to a message
    pub fn decode_block(block: Block<DefaultParams>) -> Result<Self, DecodeError> {
        let ipld = block.decode::<DagCborCodec, Ipld>()?;
        let msg = from_ipld(ipld)?;
        Ok(msg)
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
