use dwn::{
    message::{
        builder::MessageBuilder,
        descriptor::{Filter, RecordsQuery, RecordsWrite},
    },
    store::SurrealDB,
    util::DidKey,
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

    // Generate a DID.
    let did_key = DidKey::new().expect("Failed to generate DID key");
    info!("DID: {}", did_key.did);

    // Write a record.
    {
        let message = MessageBuilder::new(RecordsWrite::default())
            .authorize(did_key.kid, &did_key.jwk)
            .build()
            .expect("Failed to build message");

        // Process the message.
        let reply = dwn
            .process_message(&did_key.did, message)
            .await
            .expect("Failed to handle message");

        info!("RecordsWrite reply: {:?}", reply);
    }

    // Query the records.
    {
        // Filter the query to only include records authored by our DID.
        let message = MessageBuilder::new(RecordsQuery::new(Filter {
            attester: Some(did_key.did.clone()),
            ..Default::default()
        }))
        .build()
        .expect("Failed to build message");

        // Process the message.
        let reply = dwn
            .process_message(&did_key.did, message)
            .await
            .expect("Failed to handle message");

        info!("RecordsQuery reply: {:#?}", reply);
    }
}
