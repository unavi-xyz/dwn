use anyhow::Result;
use didkit::{
    ssi::{did::Resource, jwk::Algorithm, jws::Header},
    ResolutionInputMetadata, VerificationRelationship, DIDURL, DID_METHODS, JWK,
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
    pub fn generate_record_id(&self) -> Result<String> {
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
    pub fn new<T: Serialize>(descriptor: &T) -> Result<Self> {
        let serialized = serde_ipld_dagcbor::to_vec(descriptor)?;
        let descriptor_cid = cid_from_bytes(DagCborCodec.into(), &serialized).to_string();
        Ok(RecordIdGenerator { descriptor_cid })
    }

    pub fn generate(&self) -> Result<String> {
        let bytes = serde_ipld_dagcbor::to_vec(self)?;
        let cid = cid_from_bytes(DagCborCodec.into(), &bytes);
        Ok(cid.to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Authorization(String);

impl Authorization {
    pub async fn encode(
        algorithm: Algorithm,
        payload: &AuthPayload,
        key: &JWK,
        key_url: &DIDURL,
    ) -> Result<Self> {
        let payload = serde_json::to_string(payload)?;
        let header = Header {
            algorithm,
            key_id: Some(key_url.to_string()),
            ..Default::default()
        };
        let jws = didkit::ssi::jws::encode_sign_custom_header(payload.as_str(), key, &header)?;
        Ok(Authorization(jws))
    }

    /// Decodes the JWS and verifies the signature.
    pub async fn decode(&self) -> Result<(Header, AuthPayload)> {
        let (header, _) = didkit::ssi::jws::decode_unverified(&self.0)?;

        let key_id = match header.key_id {
            Some(key_id) => key_id,
            None => return Err(anyhow::anyhow!("header is missing key_id")),
        };
        let key_url = DIDURL::try_from(key_id)?;
        let key = url_to_key(&key_url).await?;

        let (header, payload) = didkit::ssi::jws::decode_verify(&self.0, &key)?;
        let payload = serde_json::from_slice::<AuthPayload>(payload.as_slice())?;

        Ok((header, payload))
    }
}

/// Resolves a DIDURL to a JWK.
async fn url_to_key(url: &DIDURL) -> Result<JWK> {
    let url_string = url.to_string();

    let method = match DID_METHODS.get_method(&url_string) {
        Ok(method) => method,
        Err(e) => {
            return Err(anyhow::anyhow!("did method not found: {}", e));
        }
    };

    let (_, document, _) = method
        .to_resolver()
        .resolve(&url.did, &ResolutionInputMetadata::default())
        .await;

    let document = match document {
        Some(document) => document,
        None => return Err(anyhow::anyhow!("document not found")),
    };

    let auth_ids =
        match document.get_verification_method_ids(VerificationRelationship::Authentication) {
            Ok(auth_ids) => auth_ids,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "failed to get authentication methods: {}",
                    e
                ));
            }
        };

    if !auth_ids.contains(&url_string) {
        return Err(anyhow::anyhow!(
            "key_id not found in authentication methods"
        ));
    }

    let resource = document.select_object(url)?;

    let key = match resource {
        Resource::VerificationMethod(vm) => vm.public_key_jwk,
        _ => return Err(anyhow::anyhow!("resource is not a verification method")),
    };

    let key = match key {
        Some(key) => key,
        None => return Err(anyhow::anyhow!("public key not found")),
    };

    Ok(key)
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
