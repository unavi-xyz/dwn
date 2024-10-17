use ipld_core::cid::Cid;
use rust_unixfs::file::adder::FileAdder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Data {
    Base64(String),
    Encrypted(()),
}

/// Returns a stringified CIDv1 of the data root after unixfs encoding.
pub fn compute_data_cid(data: &[u8]) -> Option<String> {
    let blocks = data_to_unixfs(data);
    blocks.last().map(|(c, _)| c.to_string())
}

fn data_to_unixfs(data: &[u8]) -> Vec<(Cid, Vec<u8>)> {
    let mut adder = FileAdder::default();

    let mut total = 0;
    let mut blocks = Vec::default();

    while total < data.len() {
        let (new_blocks, consumed) = adder.push(&data[total..]);
        total += consumed;

        for b in new_blocks {
            blocks.push(b);
        }
    }

    for b in adder.finish() {
        blocks.push(b);
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_data_cid() {
        let data = "test data".as_bytes();
        assert!(compute_data_cid(data).is_some());
    }
}
