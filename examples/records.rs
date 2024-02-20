use dwn::{
    handlers::Reply,
    message::{
        builder::MessageBuilder,
        data::Data,
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

    info!("RecordsWrite status: {:?}", reply.status());

    // Write to and update the record.
    {
        let message2 = MessageBuilder::new(RecordsWrite::default())
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .data(Data::Base64("Goodbye, world!".to_string()))
            .build()
            .expect("Failed to build message");

        let message3 = MessageBuilder::new_commit(&message2)
            .expect("Failed to create commit message")
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .build()
            .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message2)
            .await
            .expect("Failed to handle message");

        info!("RecordsWrite status: {:?}", reply.status());

        let reply = dwn
            .process_message(&did_key.did, message3)
            .await
            .expect("Failed to handle message");

        info!("RecordsCommit status: {:?}", reply.status());
    }

    // Read the record.
    {
        // TODO
    }

    // Query all messages.
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

        match reply {
            Reply::RecordsQuery(reply) => {
                info!(
                    "RecordsQuery status: {:?}, num entries: {:?}",
                    reply.status,
                    reply.entries.len()
                );
            }
            _ => panic!("Unexpected reply"),
        };
    }
}
