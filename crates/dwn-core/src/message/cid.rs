use std::collections::TryReserveError;

use ipld_core::cid::{multihash::Multihash, Cid};
use serde::Serialize;
use serde_ipld_dagcbor::EncodeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CidGenerationError {
    #[error("failed to DAG-CBOR encoding: {0}")]
    Encode(#[from] EncodeError<TryReserveError>),
    #[error("failed to parse multihash: {0}")]
    Multihash(#[from] ipld_core::cid::multihash::Error),
}

/// Returns the stringified CID of the provided struct after DAG-CBOR serialization.
pub fn compute_cid<T: Serialize>(value: &T) -> Result<String, CidGenerationError> {
    let encoded = serde_ipld_dagcbor::to_vec(value)?;
    let multihash = Multihash::from_bytes(&encoded)?;
    let cid = Cid::new_v1(DAG_CBOR_CODEC, multihash);
    Ok(cid.to_string())
}

const DAG_CBOR_CODEC: u64 = 0x71;
