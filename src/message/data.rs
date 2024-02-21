use libipld::{ipld, pb::DagPbCodec, Cid, Ipld};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    codec::Codec,
    multihash::{Code, MultihashDigest},
    serde::to_ipld,
};
use serde::{Deserialize, Serialize};

use crate::util::EncodeError;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Data {
    Base64(String),
    Encrypted(EncryptedData),
}

impl Data {
    /// Returns the CID of the data after DAG-PB encoding.
    pub fn cid(&self) -> Result<Cid, EncodeError> {
        match self {
            Data::Base64(data) => {
                let ipld = to_ipld(data)?;
                dag_pb_cid(ipld)
            }
            Data::Encrypted(_data) => {
                todo!()
            }
        }
    }
}

impl From<Data> for Vec<u8> {
    fn from(data: Data) -> Vec<u8> {
        match data {
            Data::Base64(data) => data.as_bytes().to_vec(),
            Data::Encrypted(_data) => todo!(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EncryptedData {
    pub protected: String,
    pub recipients: Vec<String>,
    pub ciphertext: String,
    pub iv: String,
    pub tag: String,
}

/// Returns the CID of the given IPLD after DAG-PB encoding.
fn dag_pb_cid(ipld: Ipld) -> Result<Cid, EncodeError> {
    let ipld = make_pb_compatible(ipld)?;
    let bytes = DagPbCodec.encode(&ipld)?;
    let hash = Code::Sha2_256.digest(&bytes);
    Ok(Cid::new_v1(DagPbCodec.into(), hash))
}

/// Converts the given IPLD into a format compatible with the DAG-PB codec.
fn make_pb_compatible(ipld: Ipld) -> Result<Ipld, EncodeError> {
    let mut data = None;
    let mut links = Vec::new();

    match ipld {
        Ipld::Link(cid) => {
            links.push(ipld!({
                "Hash": cid,
            }));
        }
        Ipld::List(list) => {
            for ipld in list {
                let cid = dag_pb_cid(ipld)?;

                links.push(ipld!({
                    "Hash": cid,
                }));
            }
        }
        Ipld::Map(map) => {
            for (key, value) in map {
                let cid = dag_pb_cid(value)?;

                links.push(ipld!({
                    "Hash": cid,
                    "Name": key,
                }));
            }
        }
        _ => data = Some(DagCborCodec.encode(&ipld)?),
    };

    match data {
        Some(data) => Ok(ipld!({
            "Data": data,
            "Links": links,
        })),
        None => Ok(ipld!({
            "Links": links,
        })),
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
