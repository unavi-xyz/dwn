use didkit::{
    ssi::{jwk::Algorithm, jws::Header},
    JWK,
};
use libipld_cbor::DagCborCodec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::util::cid_from_bytes;

use super::descriptor::Descriptor;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Message {
    #[serde(rename = "recordId")]
    pub record_id: String,
    pub descriptor: Descriptor,
    pub authorization: Option<Authorization>,
}

impl Message {
    pub fn new<T: Serialize + Into<Descriptor>>(descriptor: T) -> Self {
        let mut msg = Message {
            record_id: "".to_string(),
            descriptor: descriptor.into(),
            authorization: None,
        };

        msg.record_id = msg.generate_record_id().unwrap();

        msg
    }

    /// Returns the generated record ID for the message.
    pub fn generate_record_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        let generator = RecordIdGenerator::new(&self.descriptor)?;
        generator.generate()
    }
}

#[derive(Serialize)]
struct RecordIdGenerator {
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
}

impl RecordIdGenerator {
    pub fn new<T: Serialize>(descriptor: &T) -> Result<Self, Box<dyn std::error::Error>> {
        let serialized = serde_ipld_dagcbor::to_vec(descriptor)?;
        let descriptor_cid = cid_from_bytes(DagCborCodec.into(), &serialized).to_string();
        Ok(RecordIdGenerator { descriptor_cid })
    }

    pub fn generate(&self) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = serde_ipld_dagcbor::to_vec(self)?;
        let cid = cid_from_bytes(DagCborCodec.into(), &bytes);
        Ok(cid.to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Authorization(String);

impl Authorization {
    pub fn encode(
        algorithm: Algorithm,
        payload: &AuthPayload,
        key: &JWK,
        did: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let payload = serde_json::to_string(payload)?;
        let header = Header {
            algorithm,
            key_id: Some(did),
            ..Default::default()
        };
        let jws = didkit::ssi::jws::encode_sign_custom_header(payload.as_str(), key, &header)?;
        Ok(Authorization(jws))
    }

    pub fn decode(&self, key: &JWK) -> Result<(Header, AuthPayload), Box<dyn std::error::Error>> {
        let (header, payload) = didkit::ssi::jws::decode_verify(&self.0, key)?;
        let payload = serde_json::from_slice(payload.as_slice())?;
        Ok((header, payload))
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AuthPayload {
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
    #[serde(rename = "attestationCid")]
    pub attestation_cid: Option<String>,
    #[serde(rename = "permissionsGrantCid")]
    pub permissions_grant_cid: Option<String>,
}
