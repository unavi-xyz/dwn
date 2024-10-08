use dwn::{
    actor::Actor,
    message::descriptor::{
        protocols::{ProtocolDefinition, ProtocolsFilter},
        Descriptor,
    },
    store::SurrealStore,
    DWN,
};
use semver::Version;
use surrealdb::{engine::local::Mem, Surreal};
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_protocol_name_query() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = DWN::from(store);

    let actor = Actor::new_did_key(dwn).unwrap();

    let definition_1 = ProtocolDefinition {
        protocol: "test-protocol-1".to_string(),
        published: true,
        ..Default::default()
    };

    let register_1 = actor
        .register_protocol(definition_1.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(register_1.status.code, 200);

    let definition_2 = ProtocolDefinition {
        protocol: "test-protocol-2".to_string(),
        published: true,
        ..Default::default()
    };

    let register_2 = actor
        .register_protocol(definition_2.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(register_2.status.code, 200);

    // Filter 1.
    let filter = ProtocolsFilter {
        protocol: "test-protocol-1".to_string(),
        versions: Vec::new(),
    };

    let query = actor.query_protocols(filter).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let descriptor = match &query.entries[0].descriptor {
        Descriptor::ProtocolsConfigure(descriptor) => descriptor,
        _ => panic!("unexpected descriptor"),
    };

    assert_eq!(
        descriptor.definition.as_ref().unwrap().protocol,
        definition_1.protocol
    );

    // Filter 2.
    let filter = ProtocolsFilter {
        protocol: "test-protocol-2".to_string(),
        versions: Vec::new(),
    };

    let query = actor.query_protocols(filter).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let descriptor = match &query.entries[0].descriptor {
        Descriptor::ProtocolsConfigure(descriptor) => descriptor,
        _ => panic!("unexpected descriptor"),
    };

    assert_eq!(
        descriptor.definition.as_ref().unwrap().protocol,
        definition_2.protocol
    );

    // Filter non-existent.
    let filter = ProtocolsFilter {
        protocol: "test-protocol-3".to_string(),
        versions: Vec::new(),
    };

    let query = actor.query_protocols(filter).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);
}

#[tokio::test]
#[traced_test]
async fn test_protocol_version_query() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = DWN::from(store);

    let actor = Actor::new_did_key(dwn).unwrap();

    let definition = ProtocolDefinition {
        protocol: "test-protocol".to_string(),
        published: true,
        ..Default::default()
    };

    let register_1 = actor
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 1, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(register_1.status.code, 200);

    let register_2 = actor
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 2, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(register_2.status.code, 200);

    // Filter 0.1.0.
    let filter = ProtocolsFilter {
        protocol: definition.protocol.clone(),
        versions: vec![Version::new(0, 1, 0)],
    };

    let query = actor.query_protocols(filter).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let descriptor = match &query.entries[0].descriptor {
        Descriptor::ProtocolsConfigure(descriptor) => descriptor,
        _ => panic!("unexpected descriptor"),
    };

    assert_eq!(
        descriptor.definition.as_ref().unwrap().protocol,
        definition.protocol
    );
    assert_eq!(descriptor.protocol_version, Version::new(0, 1, 0));

    // Filter 0.2.0.
    let filter = ProtocolsFilter {
        protocol: definition.protocol.clone(),
        versions: vec![Version::new(0, 2, 0)],
    };

    let query = actor.query_protocols(filter).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let descriptor = match &query.entries[0].descriptor {
        Descriptor::ProtocolsConfigure(descriptor) => descriptor,
        _ => panic!("unexpected descriptor"),
    };

    assert_eq!(
        descriptor.definition.as_ref().unwrap().protocol,
        definition.protocol
    );
    assert_eq!(descriptor.protocol_version, Version::new(0, 2, 0));

    // Filter both.
    let filter = ProtocolsFilter {
        protocol: definition.protocol.clone(),
        versions: vec![Version::new(0, 1, 0), Version::new(0, 2, 0)],
    };

    let query = actor.query_protocols(filter).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 2);

    let descriptor = match &query.entries[0].descriptor {
        Descriptor::ProtocolsConfigure(descriptor) => descriptor,
        _ => panic!("unexpected descriptor"),
    };

    assert_eq!(
        descriptor.definition.as_ref().unwrap().protocol,
        definition.protocol
    );
    assert_eq!(descriptor.protocol_version, Version::new(0, 2, 0));

    let descriptor = match &query.entries[1].descriptor {
        Descriptor::ProtocolsConfigure(descriptor) => descriptor,
        _ => panic!("unexpected descriptor"),
    };

    assert_eq!(
        descriptor.definition.as_ref().unwrap().protocol,
        definition.protocol
    );
    assert_eq!(descriptor.protocol_version, Version::new(0, 1, 0));

    // Filter any version.
    let filter = ProtocolsFilter {
        protocol: definition.protocol.clone(),
        versions: Vec::new(),
    };

    let query = actor.query_protocols(filter).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 2);

    let descriptor = match &query.entries[0].descriptor {
        Descriptor::ProtocolsConfigure(descriptor) => descriptor,
        _ => panic!("unexpected descriptor"),
    };

    assert_eq!(
        descriptor.definition.as_ref().unwrap().protocol,
        definition.protocol
    );
    assert_eq!(descriptor.protocol_version, Version::new(0, 2, 0));

    let descriptor = match &query.entries[1].descriptor {
        Descriptor::ProtocolsConfigure(descriptor) => descriptor,
        _ => panic!("unexpected descriptor"),
    };

    assert_eq!(
        descriptor.definition.as_ref().unwrap().protocol,
        definition.protocol
    );
    assert_eq!(descriptor.protocol_version, Version::new(0, 1, 0));

    // Filter non-existent version.
    let filter = ProtocolsFilter {
        protocol: definition.protocol.clone(),
        versions: vec![Version::new(0, 3, 0)],
    };

    let query = actor.query_protocols(filter).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);
}
