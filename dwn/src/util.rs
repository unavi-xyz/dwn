use cid::Cid;

use multihash_codetable::{Code, MultihashDigest};

pub fn cid_from_bytes(codec: u64, bytes: &[u8]) -> Cid {
    let hash = Code::Sha2_256.digest(bytes);
    Cid::new_v1(codec, hash)
}
