use didkit::{ssi::jwk::Algorithm, Source, DID_METHODS, JWK};
use dwn::request::{
    descriptor::records::RecordsWrite,
    message::{AuthPayload, Authorization, Message},
};

fn main() {
    tracing_subscriber::fmt::init();

    // Generate key pair
    let key = JWK::generate_ed25519().expect("failed to generate key");

    // Create RecordsWrite message
    let mut msg = Message::new(RecordsWrite::default());

    let payload = AuthPayload {
        descriptor_cid: msg.descriptor.cid().to_string(),
        attestation_cid: None,
        permissions_grant_cid: None,
    };

    let did = DID_METHODS
        .get_method("did:key")
        .expect("did:key method not found")
        .generate(&Source::Key(&key))
        .expect("failed to generate did");

    msg.authorization = Some(
        Authorization::encode(Algorithm::EdDSA, &payload, &key, did.clone())
            .expect("failed to encode authorization"),
    );

    // Serialize message
    let serialized = serde_json::to_string(&msg).expect("failed to serialize message");

    // Deserialize message
    let msg: Message = serde_json::from_str(&serialized).expect("failed to deserialize message");

    let (header, payload) = msg
        .authorization
        .expect("authorization not found")
        .decode(&key)
        .expect("failed to decode authorization");

    assert_eq!(header.key_id, Some(did));
    assert_eq!(payload.descriptor_cid, msg.descriptor.cid().to_string());
}
