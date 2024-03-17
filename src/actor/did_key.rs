use didkit::{ssi::jwk::Algorithm, DIDMethod, Source, JWK};
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
