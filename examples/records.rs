use didkit::{ssi::jwk::Algorithm, DIDMethod, Source, JWK};
use dwn::{
    message::{
        descriptor::{Descriptor, RecordsWrite},
        Message,
    },
    store::SurrealDB,
    DWN,
};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create a DWN, using an embedded SurrealDB instance as both the data and message store.
    let db = SurrealDB::new().await.expect("Failed to create SurrealDB");
    let dwn = DWN {
        data_store: db.clone(),
        message_store: db,
    };

    // Generate a JWK and DID.
    let mut jwk = JWK::generate_ed25519().expect("Failed to generate JWK");
    jwk.algorithm = Some(Algorithm::EdDSA);

    let did = did_method_key::DIDKey
        .generate(&Source::Key(&jwk))
        .expect("Failed to generate DID");

    info!("DID: {}", did);

    // Create a RecordsWrite message.
    let mut message = Message {
        attestation: None,
        authorization: None,
        data: None,
        descriptor: Descriptor::RecordsWrite(RecordsWrite::default()),
        record_id: None,
    };

    // Authorize the message using our JWK.
    // `kid` is the DID URL that identifies the key within our DID document.
    let id = did.clone();
    let id = id.strip_prefix("did:key:").unwrap();
    let kid = format!("{}#{}", did, id);

    message
        .authorize(kid, &jwk)
        .expect("Failed to authorize message");

    // Process the message.
    let reply = dwn
        .process_message(&did, message)
        .await
        .expect("Failed to handle message");

    info!("Reply: {:?}", reply);
}
