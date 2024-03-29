use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use libipld::Cid;

use libipld_core::serde::to_ipld;

use serde::{Deserialize, Serialize};

use crate::encode::EncodeError;

use self::cid::dag_pb_cid;

mod cid;
mod encrypted;

pub use encrypted::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Data {
    Base64(String),
    Encrypted(EncryptedData),
}

impl Data {
    pub fn new_base64(data: &[u8]) -> Self {
        Data::Base64(URL_SAFE_NO_PAD.encode(data))
    }

    /// Returns the CID of the data after DAG-PB encoding.
    pub fn cid(&self) -> Result<Cid, EncodeError> {
        match self {
            Data::Base64(data) => {
                let ipld = to_ipld(data)?;
                dag_pb_cid(ipld)
            }
            Data::Encrypted(data) => {
                let ipld = to_ipld(data)?;
                dag_pb_cid(ipld)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_cid() {
        let data = Data::Base64("Hello, world!".to_string());
        data.cid().ok();
    }
}
