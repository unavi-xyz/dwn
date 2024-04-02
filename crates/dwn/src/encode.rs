use libipld::{Block, DefaultParams};
use libipld_cbor::DagCborCodec;
use libipld_core::{error::SerdeError, multihash::Code, serde::to_ipld};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error(transparent)]
    Serde(#[from] SerdeError),
    #[error("Failed to encode block: {0}")]
    Encode(anyhow::Error),
}

/// Encodes data to a DAG-CBOR block.
pub fn encode_cbor(data: &impl Serialize) -> Result<Block<DefaultParams>, EncodeError> {
    let ipld = to_ipld(data)?;
    let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &ipld)
        .map_err(EncodeError::Encode)?;
    Ok(block)
}
