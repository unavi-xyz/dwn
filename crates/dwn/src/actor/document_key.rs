use jose_jwk::jose_jwa::Signing;
use xdid::{
    core::did_url::DidUrl,
    methods::key::{DidKeyPair, PublicKey, Signer, p256::P256KeyPair, p384::P384KeyPair},
};

/// A key that is stored in the DID document.
pub struct DocumentKey {
    pub alg: Signing,
    pub key: Box<dyn Signer + Send + Sync>,
    /// URL to the key.
    pub url: DidUrl,
}

impl DocumentKey {
    pub fn from_did_key(alg: Signing, key: impl DidKeyPair + Send + Sync + 'static) -> Self {
        let did = key.public().to_did();
        let fragment = did
            .to_string()
            .strip_prefix("did:key:")
            .unwrap()
            .to_string();

        let url = DidUrl {
            did,
            fragment: Some(fragment),
            path_abempty: None,
            query: None,
        };

        Self {
            alg,
            key: Box::new(key),
            url,
        }
    }
}

impl From<P256KeyPair> for DocumentKey {
    fn from(value: P256KeyPair) -> Self {
        Self::from_did_key(Signing::Es256, value)
    }
}

impl From<P384KeyPair> for DocumentKey {
    fn from(value: P384KeyPair) -> Self {
        Self::from_did_key(Signing::Es384, value)
    }
}
