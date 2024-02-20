use dwn::{
    message::{
        builder::MessageBuilder,
        data::Data,
        descriptor::{Filter, RecordsCommit, RecordsQuery, RecordsWrite},
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

    // Create a record.
    let message1 = MessageBuilder::new(RecordsWrite::default())
        .authorize(did_key.kid.clone(), &did_key.jwk)
        .data(Data::Base64("Hello, world!".to_string()))
        .build()
        .expect("Failed to build message");

    let reply = dwn
        .process_message(&did_key.did, message1)
        .await
        .expect("Failed to handle message");

    info!("RecordsWrite reply: {:?}", reply);

    // Write to and update the record.
    {
        let message2 = MessageBuilder::new(RecordsWrite::default())
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .data(Data::Base64("Goodbye, world!".to_string()))
            .build()
            .expect("Failed to build message");

        let entry_id = message2
            .generate_record_id()
            .expect("Failed to generate record ID");

        let reply = dwn
            .process_message(&did_key.did, message2)
            .await
            .expect("Failed to handle message");

        info!("RecordsWrite reply: {:?}", reply);

        let message3 = MessageBuilder::new(RecordsCommit::new(entry_id))
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .build()
            .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message3)
            .await
            .expect("Failed to handle message");

        info!("RecordsCommit reply: {:?}", reply);
    }

    // Read the record.
    {
        // TODO
    }

    // Query messages.
    {
        let message4 = MessageBuilder::new(RecordsQuery::new(Filter {
            attester: Some(did_key.did.clone()),
            ..Default::default()
        }))
        .build()
        .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message4)
            .await
            .expect("Failed to handle message");

        info!("RecordsQuery reply: {:#?}", reply);
    }
}
