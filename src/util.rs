use didkit::{ssi::jwk::Algorithm, DIDMethod, Source, JWK};
use libipld::{Block, DefaultParams, Ipld};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    error::SerdeError,
    multihash::Code,
    serde::{from_ipld, to_ipld},
};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DidKeygenError {
    #[error("Failed to generate JWK: {0}")]
    Jwk(#[from] didkit::ssi::jwk::Error),
    #[error("Failed to generate DID")]
    DidGen,
}

pub struct DidKey {
    pub did: String,
    pub jwk: JWK,
    // Verifiable Credential DID URL for the key.
    pub kid: String,
}

impl DidKey {
    /// Generates a did:key.
    pub fn new() -> Result<Self, DidKeygenError> {
        let mut jwk = JWK::generate_ed25519()?;
        jwk.algorithm = Some(Algorithm::EdDSA);

        let did = did_method_key::DIDKey
            .generate(&Source::Key(&jwk))
            .ok_or(DidKeygenError::DidGen)?;

        let id = did.strip_prefix("did:key:").ok_or(DidKeygenError::DidGen)?;
        let kid = format!("{}#{}", did, id);

        Ok(DidKey { did, jwk, kid })
    }
}

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("Failed to serialize/deserialize IPLD: {0}")]
    Serde(#[from] SerdeError),
    #[error("Failed to encode/decode CBOR: {0}")]
    Encode(#[from] anyhow::Error),
}

/// Encodes data to a DAG-CBOR block.
pub fn encode_cbor(data: &impl Serialize) -> Result<Block<DefaultParams>, EncodeError> {
    let ipld = to_ipld(data)?;
    let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &ipld)?;
    Ok(block)
}

/// Decodes a DAG-CBOR block.
pub fn decode_block<T: DeserializeOwned>(block: Block<DefaultParams>) -> Result<T, EncodeError> {
    let ipld = block.decode::<DagCborCodec, Ipld>()?;
    let data = from_ipld(ipld)?;
    Ok(data)
}
