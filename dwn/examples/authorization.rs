use did_method_key::DIDKey;
use didkit::{ssi::jwk::Algorithm, DIDMethod, Source, JWK};
use dwn::request::{
    descriptor::records::RecordsWrite,
    message::{AuthPayload, Authorization, Message},
};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    // Generate a did:key
    let key = JWK::generate_ed25519().expect("failed to generate key");
    let source = Source::Key(&key);
    let did = DIDKey.generate(&source).expect("failed to generate DID");

    info!("DID: {}", did);

    // Get DID URL to the verification method
    let fragment = did.clone().strip_prefix("did:key:").unwrap().to_string();
    let key_id = format!("{}#{}", did, fragment);

    // Create RecordsWrite message
    let mut msg = Message::new(RecordsWrite::default());

    let payload = AuthPayload {
        descriptor_cid: msg.descriptor.cid().to_string(),
        attestation_cid: None,
        permissions_grant_cid: None,
    };

    msg.authorization = Some(
        Authorization::encode(Algorithm::EdDSA, &payload, &key, key_id.clone())
            .await
            .expect("failed to encode authorization"),
    );

    // Serialize message
    let serialized = serde_json::to_string(&msg).expect("failed to serialize message");

    // Deserialize message
    let msg = serde_json::from_str::<Message>(&serialized).expect("failed to deserialize message");

    // Validate message
    let (header, payload) = msg
        .authorization
        .expect("message is missing authorization")
        .decode_verify()
        .await
        .expect("failed to decode authorization");

    assert_eq!(header.key_id, Some(key_id));
    assert_eq!(payload.descriptor_cid, msg.descriptor.cid().to_string());
}
