use did_method_key::DIDKey;
use didkit::{DIDMethod, Source, JWK};
use tracing::debug;

pub fn gen_did() -> (String, JWK) {
    let key = JWK::generate_ed25519().expect("failed to generate key");
    let source = Source::Key(&key);
    let did = DIDKey.generate(&source).expect("failed to generate did");

    debug!("Generated DID: {}", did);

    (did, key)
}
