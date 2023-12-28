use didkit::{ssi::jwk::Algorithm, Source, DIDURL, DID_METHODS, JWK};
use dwn::request::{
    descriptor::records::RecordsWrite,
    message::{AuthPayload, Authorization, Message},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create RecordsWrite message
    let msg = {
        let key = JWK::generate_ed25519().expect("failed to generate key");

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
        let mut key_url = DIDURL::try_from(did.clone()).expect("failed to parse did url");
        let did_hash = did.split(':').nth(2).expect("failed to get did body");
        key_url.fragment = Some(did_hash.to_string());

        msg.authorization = Some(
            Authorization::encode(Algorithm::EdDSA, &payload, &key, &key_url)
                .await
                .expect("failed to encode authorization"),
        );

        msg
    };

    // Serialize message
    let serialized = serde_json::to_string(&msg).expect("failed to serialize message");

    // Deserialize message
    let msg = serde_json::from_str::<Message>(&serialized).expect("failed to deserialize message");

    // Validate message
    msg.authorization
        .expect("message is missing authorization")
        .decode()
        .await
        .expect("failed to decode authorization");
}
