use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};

const RAW: u64 = 0x55;

pub fn cid_from_bytes(bytes: &[u8]) -> Cid {
    let hash = Code::Sha2_256.digest(bytes);
    Cid::new_v1(RAW, hash)
}
