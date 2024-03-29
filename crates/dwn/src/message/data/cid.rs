use libipld::{ipld, pb::DagPbCodec, Cid, Ipld};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    codec::Codec,
    multihash::{Code, MultihashDigest},
};

use crate::encode::EncodeError;

/// Returns the CID of the given IPLD after DAG-PB encoding.
pub fn dag_pb_cid(ipld: Ipld) -> Result<Cid, EncodeError> {
    let ipld = make_pb_compatible(ipld)?;
    let bytes = DagPbCodec.encode(&ipld).map_err(EncodeError::Encode)?;
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
        _ => data = Some(DagCborCodec.encode(&ipld).map_err(EncodeError::Encode)?),
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
    use libipld::ipld;

    use super::*;

    #[test]
    fn test_list_encode() {
        let ipld = ipld!([1, 2, 3]);
        let cid = dag_pb_cid(ipld).unwrap();
        assert!(!cid.to_bytes().is_empty());
    }

    #[test]
    fn test_map_encode() {
        let ipld = ipld!({
            "a": 1,
            "b": 2,
            "c": 3,
        });
        let cid = dag_pb_cid(ipld).unwrap();
        assert!(!cid.to_bytes().is_empty());
    }

    #[test]
    fn test_nested_encode() {
        let ipld = ipld!({
            "a": [1, 2, 3],
            "b": {
                "c": 4,
                "d": [
                    {
                        "e": 5,
                    },
                ],
            },
        });
        let cid = dag_pb_cid(ipld).unwrap();
        assert!(!cid.to_bytes().is_empty());
    }
}
