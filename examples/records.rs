use dwn::{
    handlers::Reply,
    message::{
        builder::MessageBuilder,
        data::Data,
        descriptor::{
            Filter, RecordsCommit, RecordsDelete, RecordsQuery, RecordsRead, RecordsWrite,
        },
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
    let message1 = MessageBuilder::new::<RecordsWrite>()
        .authorize(did_key.kid.clone(), &did_key.jwk)
        .data(Data::Base64("Hello, world!".to_string()))
        .build()
        .expect("Failed to build message");

    let record_id = message1.record_id.clone();

    let reply = dwn
        .process_message(&did_key.did, message1)
        .await
        .expect("Failed to handle message");

    info!("RecordsWrite status: {:?}", reply.status());

    // Read the record.
    {
        let message2 = MessageBuilder::from_descriptor(RecordsRead::new(record_id.clone()))
            .build()
            .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message2)
            .await
            .expect("Failed to handle message");

        match reply {
            Reply::RecordsRead(reply) => {
                info!("RecordsRead status: {:?}", reply.status);
                info!("RecordsRead data: {:?}", reply.data);
            }
            _ => panic!("Unexpected reply"),
        };
    }

    // Write to and update the record.
    {
        let message2 = MessageBuilder::new::<RecordsWrite>()
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .data(Data::Base64("Goodbye, world!".to_string()))
            .build()
            .expect("Failed to build message");

        let message3 = MessageBuilder::new::<RecordsCommit>()
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .parent(&message2)
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

    // Query all messages.
    {
        let message2 = MessageBuilder::from_descriptor(RecordsQuery::new(Filter {
            attester: Some(did_key.did.clone()),
            ..Default::default()
        }))
        .build()
        .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message2)
            .await
            .expect("Failed to handle message");

        match reply {
            Reply::RecordsQuery(reply) => {
                info!("RecordsQuery status: {:?}", reply.status);
                info!("RecordsQuery number of entries: {:?}", reply.entries.len());
            }
            _ => panic!("Unexpected reply"),
        };
    }

    // Delete the record.
    {
        let message2 = MessageBuilder::new::<RecordsDelete>()
            .authorize(did_key.kid.clone(), &did_key.jwk)
            .record_id(Some(record_id.clone()))
            .build()
            .expect("Failed to build message");

        let reply = dwn
            .process_message(&did_key.did, message2)
            .await
            .expect("Failed to handle message");

        info!("RecordsDelete status: {:?}", reply.status());
    }
}
