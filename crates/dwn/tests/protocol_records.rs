use std::sync::Arc;

use dwn::{
    actor::Actor,
    message::descriptor::{
        protocols::{
            Action, ActionCan, ActionWho, ProtocolDefinition, ProtocolStructure, StructureType,
        },
        records::RecordsFilter,
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
        .data(data)
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
    let query = alice
        .query_records(RecordsFilter {
            protocol: Some("test-protocol".to_string()),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
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

    // Alice can read chats.
    let query = alice
        .query_records(RecordsFilter {
            protocol: Some("test-protocol".to_string()),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let entry = query.entries.get(0).unwrap();
}
