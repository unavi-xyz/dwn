use std::collections::BTreeMap;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use iana_media_types::{Application, MediaType};
use libipld_cbor::DagCborCodec;
use libipld_core::{codec::Codec, ipld::Ipld};
use libipld_json::DagJsonCodec;
use libipld_pb::DagPbCodec;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::util::cid_from_bytes;

pub trait Data {
    /// Returns the data as a base64url-encoded string.
    fn to_base64url(&self) -> String;
    /// Returns the data as an IPLD object.
    fn to_ipld(&self) -> Ipld;
    /// Returns the data as a DAG-PB encoded byte array.
    fn to_pb(&self) -> Vec<u8> {
        let ipld = self.to_ipld();
        let data = ipld_to_pb(ipld);
        DagPbCodec.encode(&data).expect("Failed to encode IPLD")
    }
    /// Returns the CID of the DAG-PB encoded data.
    fn data_cid(&self) -> String {
        let pb = self.to_pb();
        let cid = cid_from_bytes(DagPbCodec.into(), &pb);
        cid.to_string()
    }
    /// Returns the data format of this data.
    fn data_format(&self) -> MediaType;
}

/// Converts an IPLD object to be DAG-PB compatible.
/// DAG-PB is for opaque binary data, so we use CBOR to encode the IPLD object as bytes.
fn ipld_to_pb(ipld: Ipld) -> Ipld {
    let mut links = Vec::<Ipld>::new();

    let data: Vec<u8> = match ipld {
        Ipld::Map(map) => {
            for (key, value) in map {
                let mut pb_link = BTreeMap::<String, Ipld>::new();
                pb_link.insert("Name".to_string(), key.into());

                let value = ipld_to_pb(value);
                let cid = cid_from_bytes(DagPbCodec.into(), &DagPbCodec.encode(&value).unwrap());
                pb_link.insert("Hash".to_string(), cid.into());

                links.push(pb_link.into());
            }

            Vec::new()
        }
        Ipld::List(list) => {
            for value in list {
                let value = ipld_to_pb(value);
                let cid = cid_from_bytes(DagPbCodec.into(), &DagPbCodec.encode(&value).unwrap());

                links.push(cid.into());
            }

            Vec::new()
        }
        Ipld::Link(cid) => {
            let mut pb_link = BTreeMap::<String, Ipld>::new();
            pb_link.insert("Hash".to_string(), cid.into());

            links.push(pb_link.into());

            Vec::new()
        }
        _ => DagCborCodec.encode(&ipld).unwrap(),
    };

    let mut pb_node = BTreeMap::<String, Ipld>::new();
    pb_node.insert("Links".to_string(), links.into());

    if !data.is_empty() {
        pb_node.insert("Data".to_string(), data.into());
    }

    pb_node.into()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonData(pub Value);

impl Data for JsonData {
    fn to_base64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0.to_string())
    }

    fn to_ipld(&self) -> Ipld {
        let json = self.0.to_string();
        let bytes = json.as_bytes();
        DagJsonCodec.decode(bytes).expect("Failed to decode JSON")
    }

    fn data_format(&self) -> MediaType {
        Application::Json.into()
    }
}

#[cfg(test)]
mod tests {
    use super::{Data, JsonData};
    use libipld_core::codec::Codec;

    #[test]
    fn test_json_data() {
        let data = JsonData(serde_json::json!({
            "foo": "bar",
        }));

        assert_eq!(data.to_base64url(), "eyJmb28iOiJiYXIifQ");
        assert_eq!(data.data_format().to_string(), "application/json");

        let ipld = data.to_ipld();
        let encoded = libipld_json::DagJsonCodec
            .encode(&ipld)
            .expect("Failed to encode IPLD");
        let encoded_string = String::from_utf8(encoded).expect("Failed to convert to string");

        assert_eq!(encoded_string, r#"{"foo":"bar"}"#);
    }
}
