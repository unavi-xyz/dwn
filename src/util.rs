use libipld::{Block, DefaultParams, Ipld};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    error::SerdeError,
    multihash::Code,
    serde::{from_ipld, to_ipld},
};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CborEncodeError {
    #[error("Failed to serialize/deserialize IPLD: {0}")]
    Serde(#[from] SerdeError),
    #[error("Failed to encode/decode CBOR: {0}")]
    Encode(#[from] anyhow::Error),
}

/// Encodes data to a DAG-CBOR block.
pub fn encode_cbor(data: &impl Serialize) -> Result<Block<DefaultParams>, CborEncodeError> {
    let ipld = to_ipld(data)?;
    let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &ipld)?;
    Ok(block)
}

/// Decodes a DAG-CBOR block.
pub fn decode_block<T: DeserializeOwned>(
    block: Block<DefaultParams>,
) -> Result<T, CborEncodeError> {
    let ipld = block.decode::<DagCborCodec, Ipld>()?;
    let data = from_ipld(ipld)?;
    Ok(data)
}
