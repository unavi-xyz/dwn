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
pub enum EncodeError {
    #[error(transparent)]
    Serde(#[from] SerdeError),
    #[error("Failed to encode block: {0}")]
    Encode(anyhow::Error),
    #[error("Failed to decode block: {0}")]
    Decode(anyhow::Error),
}

/// Encodes data to a DAG-CBOR block.
pub fn encode_cbor(data: &impl Serialize) -> Result<Block<DefaultParams>, EncodeError> {
    let ipld = to_ipld(data)?;
    let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &ipld)
        .map_err(EncodeError::Encode)?;
    Ok(block)
}

/// Decodes a DAG-CBOR block.
pub fn decode_cbor<T: DeserializeOwned>(block: Block<DefaultParams>) -> Result<T, EncodeError> {
    let ipld = block
        .decode::<DagCborCodec, Ipld>()
        .map_err(EncodeError::Decode)?;
    let data = from_ipld(ipld)?;
    Ok(data)
}
