use std::sync::Arc;

use dwn::{
    actor::{Actor, MessageBuilder},
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
use iana_media_types::Application;
use semver::Version;
use surrealdb::{engine::local::Mem, Surreal};
use tracing_test::traced_test;

const PROTOCOL: &str = "chat-protocol";
const STRUCTURE: &str = "chat";

fn chat_protocol() -> ProtocolDefinition {
    let mut definition = ProtocolDefinition {
        protocol: PROTOCOL.to_string(),
        published: true,
        ..Default::default()
    };

    definition.types.insert(
        STRUCTURE.to_string(),
        StructureType {
            data_format: vec![Application::Json.into()],
            ..Default::default()
        },
    );

    definition
        .structure
        .insert(STRUCTURE.to_string(), ProtocolStructure::default());

    definition
}

#[tokio::test]
#[traced_test]
async fn test_no_read() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn).unwrap();

    // Register protocol.
    let definition = chat_protocol();

    let register = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 1, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(register.status.code, 200);

    // Alice creates a chat.
    let data = "Hello, world!".as_bytes().to_vec();
    let record = alice
        .create_record()
        .data(data.clone())
        .data_format(Application::Json.into())
        .protocol(
            PROTOCOL.to_string(),
            Version::new(0, 1, 0),
            STRUCTURE.to_string(),
        )
        .published(true)
        .process()
        .await
        .unwrap();
    assert_eq!(record.reply.status.code, 200);

    // Alice cannot read chats.
    let filter = RecordsFilter {
        protocol: Some(PROTOCOL.to_string()),
        protocol_version: Some(Version::new(0, 1, 0)),
        ..Default::default()
    };
    let query = alice.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Bob cannot read Alice's chat.
    let query = bob
        .query_records(filter.clone())
        .target(alice.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);
}

#[tokio::test]
#[traced_test]
async fn test_anyone_read() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn).unwrap();

    // Register protocol.
    let mut definition = chat_protocol();
    definition
        .structure
        .get_mut(STRUCTURE)
        .unwrap()
        .actions
        .push(Action {
            who: ActionWho::Anyone,
            of: None,
            can: ActionCan::Read,
        });

    let register = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 1, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(register.status.code, 200);

    // Alice creates a chat.
    let data = "Hello, world!".as_bytes().to_vec();
    let record = alice
        .create_record()
        .data(data.clone())
        .data_format(Application::Json.into())
        .protocol(
            PROTOCOL.to_string(),
            Version::new(0, 1, 0),
            STRUCTURE.to_string(),
        )
        .published(true)
        .process()
        .await
        .unwrap();
    assert_eq!(record.reply.status.code, 200);

    // Alice can read chats.
    let filter = RecordsFilter {
        protocol: Some(PROTOCOL.to_string()),
        protocol_version: Some(Version::new(0, 1, 0)),
        ..Default::default()
    };
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

    // Bob can read Alice's chat.
    let query = bob
        .query_records(filter)
        .target(alice.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let entry = query.entries.first().unwrap();
    let read = bob
        .read_record(entry.record_id.clone())
        .target(alice.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));
}

#[tokio::test]
#[traced_test]
async fn test_author_read() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn).unwrap();

    // Register protocol.
    let mut definition = chat_protocol();

    let register = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 1, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(register.status.code, 200);

    // Alice creates a chat.
    let data = "Hello, world!".as_bytes().to_vec();
    let record = alice
        .create_record()
        .data(data.clone())
        .data_format(Application::Json.into())
        .protocol(
            PROTOCOL.to_string(),
            Version::new(0, 1, 0),
            STRUCTURE.to_string(),
        )
        .published(true)
        .process()
        .await
        .unwrap();
    assert_eq!(record.reply.status.code, 200);

    // Alice cannot read chats.
    let mut filter = RecordsFilter {
        protocol: Some(PROTOCOL.to_string()),
        protocol_version: Some(Version::new(0, 1, 0)),
        ..Default::default()
    };
    let query = alice.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Update the protocol to enable reads.
    definition
        .structure
        .get_mut(STRUCTURE)
        .unwrap()
        .actions
        .push(Action {
            who: ActionWho::Author,
            of: Some(STRUCTURE.to_string()),
            can: ActionCan::Read,
        });

    let update = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 2, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(update.status.code, 200);

    // Alice still cannot read 0.1.0 chats.
    let query = alice.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // There are no 0.2.0 chats to read.
    filter.protocol_version = Some(Version::new(0, 2, 0));
    let query = alice.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Update the record to 0.2.0.
    let update = alice
        .update_record(record.record_id.clone(), record.entry_id.clone())
        .data(data.clone())
        .data_format(Application::Json.into())
        .protocol(
            PROTOCOL.to_string(),
            Version::new(0, 2, 0),
            STRUCTURE.to_string(),
        )
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Alice can read 0.2.0 chats.
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

    // Bob cannot read 0.2.0 chats.
    let query = bob.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);
}

#[tokio::test]
#[traced_test]
async fn test_anyone_write_recipient_read() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn).unwrap();

    // Bob registers protocol.
    let mut definition = chat_protocol();
    let actions = &mut definition.structure.get_mut(STRUCTURE).unwrap().actions;

    actions.push(Action {
        who: ActionWho::Anyone,
        of: None,
        can: ActionCan::Write,
    });

    actions.push(Action {
        who: ActionWho::Recipient,
        of: Some(STRUCTURE.to_string()),
        can: ActionCan::Read,
    });

    let register = bob
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 1, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(register.status.code, 200);

    // Alice creates a chat for Bob.
    let data = "Hello Bob.".as_bytes().to_vec();
    let record = alice
        .create_record()
        .data(data.clone())
        .data_format(Application::Json.into())
        .protocol(
            PROTOCOL.to_string(),
            Version::new(0, 1, 0),
            STRUCTURE.to_string(),
        )
        .target(bob.did.clone())
        .published(true)
        .process()
        .await
        .unwrap();
    assert_eq!(record.reply.status.code, 200);

    // Alice cannot read chats.
    let filter = RecordsFilter {
        protocol: Some(PROTOCOL.to_string()),
        protocol_version: Some(Version::new(0, 1, 0)),
        ..Default::default()
    };
    let query = alice
        .query_records(filter.clone())
        .target(bob.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Bob can read his chat.
    let query = bob.query_records(filter.clone()).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let entry = query.entries.first().unwrap();
    let read = bob
        .read_record(entry.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));
}
