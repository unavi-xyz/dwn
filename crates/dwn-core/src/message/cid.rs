use std::collections::TryReserveError;

use ipld_core::cid::{multihash::Multihash, Cid};
use serde::Serialize;
use serde_ipld_dagcbor::EncodeError;
use sha3::{Digest, Sha3_256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CidGenerationError {
    #[error("failed to DAG-CBOR encoding: {0}")]
    Encode(#[from] EncodeError<TryReserveError>),
    #[error("failed to parse multihash: {0}")]
    Multihash(#[from] ipld_core::cid::multihash::Error),
}

/// Returns a stringified CIDv1 of the provided data after DAG-CBOR serialization.
pub fn compute_cid_cbor<T: Serialize>(value: &T) -> Result<String, CidGenerationError> {
    let encoded = serde_ipld_dagcbor::to_vec(value)?;

    let mut hasher = Sha3_256::new();
    hasher.update(&encoded);
    let hash = hasher.finalize();

    let multihash = Multihash::wrap(CODE_SHA3_256, &hash)?;
    let cid = Cid::new_v1(CODEC_DAG_CBOR, multihash);
    Ok(cid.to_string())
}

const CODEC_DAG_CBOR: u64 = 0x71;
const CODE_SHA3_256: u64 = 0x16;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[derive(Serialize)]
    struct TestData {
        hello: String,
    }

    #[test]
    fn test_cid_cbor() {
        let data = TestData {
            hello: "world".to_string(),
        };

        let cid_str = compute_cid_cbor(&data).unwrap();

        let cid = Cid::from_str(&cid_str).unwrap();
        assert_eq!(cid.codec(), CODEC_DAG_CBOR);

        let hash = cid.hash();
        assert_eq!(hash.code(), CODE_SHA3_256);
    }
}
