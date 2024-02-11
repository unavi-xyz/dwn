use std::collections::BTreeMap;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use iana_media_types::{Application, MediaType};
use libipld_core::{codec::Codec, ipld::Ipld};
use libipld_json::DagJsonCodec;
use libipld_pb::DagPbCodec;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::util::cid_from_bytes;

pub trait Data: Sized {
    /// MIME type of the data.
    fn data_format(&self) -> MediaType;

    /// Encodes an IPLD object into a byte array.
    fn encode(&self, ipld: &Ipld) -> Vec<u8>;
    /// Decodes a byte array into an IPLD object.
    fn decode(&self, bytes: &[u8]) -> Ipld;
    /// Returns the codec of the encoder.
    fn codec(&self) -> u64;

    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;

    fn to_base64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.to_bytes())
    }
    fn try_from_base64url(data: &str) -> Result<Self, base64::DecodeError> {
        let bytes = URL_SAFE_NO_PAD.decode(data.as_bytes())?;
        Ok(Self::from_bytes(&bytes))
    }

    /// Returns the CID of the DAG-PB encoded data.
    fn data_cid(&self) -> String {
        let pb = self.to_pb();
        let cid = cid_from_bytes(self.codec(), &pb);
        cid.to_string()
    }
    /// Returns the data as a DAG-PB encoded byte array.
    fn to_pb(&self) -> Vec<u8> {
        let ipld = self.decode(&self.to_bytes());
        let ipld = self.ipld_to_pb(ipld);
        DagPbCodec.encode(&ipld).expect("Failed to encode IPLD")
    }
    /// Returns a DAG-PB compatible version of an IPLD object.
    fn ipld_to_pb(&self, ipld: Ipld) -> Ipld {
        let mut links = Vec::<Ipld>::new();

        let data = match ipld {
            Ipld::Link(cid) => {
                let mut pb_link = BTreeMap::<String, Ipld>::new();
                pb_link.insert("Hash".to_string(), cid.into());

                links.push(pb_link.into());

                None
            }
            Ipld::List(list) => {
                for ipld in list {
                    let mut pb_link = BTreeMap::<String, Ipld>::new();

                    let ipld = self.ipld_to_pb(ipld);
                    let bytes = DagPbCodec.encode(&ipld).expect("Failed to encode IPLD");
                    let cid = cid_from_bytes(DagPbCodec.into(), &bytes);
                    pb_link.insert("Hash".to_string(), cid.into());

                    links.push(pb_link.into());
                }

                None
            }
            Ipld::Map(map) => {
                for (key, value) in map {
                    let mut pb_link = BTreeMap::<String, Ipld>::new();

                    let value = self.ipld_to_pb(value);
                    let bytes = DagPbCodec.encode(&value).expect("Failed to encode IPLD");
                    let cid = cid_from_bytes(DagPbCodec.into(), &bytes);
                    pb_link.insert("Hash".to_string(), cid.into());
                    pb_link.insert("Name".to_string(), key.into());

                    links.push(pb_link.into());
                }

                None
            }
            _ => Some(self.encode(&ipld)),
        };

        let mut pb_node = BTreeMap::<String, Ipld>::new();
        pb_node.insert("Links".to_string(), links.into());

        if let Some(data) = data {
            pb_node.insert("Data".to_string(), data.into());
        }

        pb_node.into()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JsonData(pub Value);

impl Data for JsonData {
    fn data_format(&self) -> MediaType {
        Application::Json.into()
    }
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_string().into_bytes()
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        let value = serde_json::from_slice(bytes).expect("Failed to parse JSON");
        JsonData(value)
    }
    fn encode(&self, ipld: &Ipld) -> Vec<u8> {
        DagJsonCodec.encode(ipld).expect("Failed to encode IPLD")
    }
    fn decode(&self, bytes: &[u8]) -> Ipld {
        DagJsonCodec.decode(bytes).expect("Failed to decode IPLD")
    }
    fn codec(&self) -> u64 {
        DagJsonCodec.into()
    }
}

#[cfg(test)]
mod tests {
    use super::{Data, JsonData};

    #[test]
    fn basic_json() {
        let data = JsonData(serde_json::json!({
            "foo": "bar",
        }));

        assert_eq!(data.data_format().to_string(), "application/json");

        let base64url = data.to_base64url();

        assert_eq!(base64url, "eyJmb28iOiJiYXIifQ");
        assert_eq!(JsonData::try_from_base64url(&base64url), Ok(data.clone()));

        let ipld = data.decode(&data.to_bytes());
        let encoded = data.encode(&ipld);
        let encoded_string = String::from_utf8(encoded).expect("Failed to convert to string");
        assert_eq!(encoded_string, data.0.to_string());

        let cid = data.data_cid();
        assert_eq!(
            cid,
            "baguqeeramtldk52dh5v5sazm6qr7xv5flgmuzlg5k2o2zr4yvscjunpnm2gq"
        )
    }
}
