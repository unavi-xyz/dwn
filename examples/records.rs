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

    // Create a message to write a record.
    let message = Message {
        attestation: None,
        authorization: None,
        data: None,
        descriptor: Descriptor::RecordsWrite(RecordsWrite::default()),
        record_id: None,
    };

    let tenant = "did:example:123";

    // Process the message.
    let reply = dwn
        .process_message(tenant, message)
        .await
        .expect("Failed to handle message");

    info!("Reply: {:?}", reply);
}
