use base64::Engine;
use iana_media_types::Application;
use libipld_core::{codec::Codec, ipld::Ipld};
use libipld_json::DagJsonCodec;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::request::DataFormat;

pub trait Data {
    /// Returns the data as a base64url-encoded string.
    fn to_base64url(&self) -> String;
    /// Returns the data as an IPLD object.
    fn to_ipld(&self) -> Ipld;
    /// Returns the data format of this data.
    fn data_format(&self) -> DataFormat;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonData(Value);

impl Data for JsonData {
    fn to_base64url(&self) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(self.0.to_string())
    }

    fn to_ipld(&self) -> Ipld {
        let json = self.0.to_string();
        let bytes = json.as_bytes();
        DagJsonCodec.decode(bytes).expect("Failed to decode JSON")
    }

    fn data_format(&self) -> DataFormat {
        DataFormat::MediaType(Application::Json.into())
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
        assert_eq!(data.data_format().to_string(), "\"application/json\"");

        let ipld = data.to_ipld();
        let encoded = libipld_json::DagJsonCodec
            .encode(&ipld)
            .expect("Failed to encode IPLD");
        let encoded_string = String::from_utf8(encoded).expect("Failed to convert to string");

        assert_eq!(encoded_string, "{\"foo\":\"bar\"}");
    }
}
