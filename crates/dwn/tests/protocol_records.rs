use std::sync::Arc;

use dwn::{
    actor::Actor,
    message::{
        descriptor::{
            protocols::{
                Action, ActionCan, ActionWho, ProtocolDefinition, ProtocolStructure, StructureType,
            },
            records::RecordsFilter,
        },
        Data,
    },
    store::SurrealStore,
    DWN,
};
use iana_media_types::{Application, MediaType};
use semver::Version;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_author_write_read() {
    let store = SurrealStore::new().await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn).unwrap();

    // Register protocol.
    let mut definition = ProtocolDefinition {
        protocol: "test-protocol".to_string(),
        published: true,
        ..Default::default()
    };

    definition.types.insert(
        "chat".to_string(),
        StructureType {
            data_format: vec![MediaType::Application(Application::Json)],
            ..Default::default()
        },
    );

    definition
        .structure
        .insert("chat".to_string(), ProtocolStructure::default());

    let register = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 1, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(register.status.code, 200);

    // Alice cannot write chats.
    let data = "Hello, world!".as_bytes().to_vec();
    let record = alice
        .create_record()
        .data(data.clone())
        .data_format(MediaType::Application(Application::Json))
        .protocol(
            "test-protocol".to_string(),
            Version::new(0, 1, 0),
            "chat".to_string(),
        )
        .published(true)
        .process()
        .await;
    assert!(record.is_err());

    // Update the protocol to enable writes.
    let mut structure = ProtocolStructure::default();
    structure.actions.push(Action {
        who: ActionWho::Author,
        of: Some("chat".to_string()),
        can: ActionCan::Write,
    });

    definition
        .structure
        .insert("chat".to_string(), structure.clone());

    let update = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 2, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(update.status.code, 200);

    // Alice can write chats.
    let record = alice
        .create_record()
        .data(data.clone())
        .data_format(MediaType::Application(Application::Json))
        .protocol(
            "test-protocol".to_string(),
            Version::new(0, 2, 0),
            "chat".to_string(),
        )
        .published(true)
        .process()
        .await
        .unwrap();
    assert_eq!(record.reply.status.code, 200);

    // Alice cannot read chats.
    let mut filter = RecordsFilter {
        protocol: Some("test-protocol".to_string()),
        protocol_version: Some(Version::new(0, 2, 0)),
        ..Default::default()
    };
    let query = alice.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Update the protocol to enable reads.
    structure.actions.push(Action {
        who: ActionWho::Author,
        of: Some("chat".to_string()),
        can: ActionCan::Read,
    });
    definition
        .structure
        .insert("chat".to_string(), structure.clone());

    let update = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 3, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(update.status.code, 200);

    // Alice still cannot read 0.2.0 chats.
    let query = alice.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // There are no 0.3.0 chats to read.
    filter.protocol_version = Some(Version::new(0, 3, 0));
    let query = alice.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Update the record to 0.3.0.
    let update = alice
        .update_record(record.record_id.clone(), record.entry_id.clone())
        .data(data.clone())
        .protocol(
            "test-protocol".to_string(),
            Version::new(0, 3, 0),
            "chat".to_string(),
        )
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Alice can read 0.3.0 chats.
    let query = alice.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let entry = query.entries.first().unwrap();
    let read = alice
        .read_record(entry.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Bob cannot read 0.3.0 chats.
    let query = bob.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);
}
