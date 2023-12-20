use libipld_core::{
    cid::Cid,
    multihash::{Code, MultihashDigest},
};

/// Generates a CID V1 from the given codec and bytes.
pub fn cid_from_bytes(codec: u64, bytes: &[u8]) -> Cid {
    let hash = Code::Sha2_256.digest(bytes);
    Cid::new_v1(codec, hash)
}
