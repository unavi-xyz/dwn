use std::sync::Arc;

use dwn::{
    actor::{Actor, MessageBuilder},
    message::descriptor::{
        protocols::{ProtocolDefinition, ProtocolsFilter},
        records::RecordsFilter,
    },
    store::SurrealStore,
    DWN,
};
use semver::Version;
use surrealdb::{engine::local::Mem, Surreal};

const UNPUBLISHED_DEFINITION: &str = include_str!("./unpublished.json");

#[tokio::test]
pub async fn test_unpublished() {
    let definition: ProtocolDefinition = serde_json::from_str(UNPUBLISHED_DEFINITION).unwrap();

    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn).unwrap();

    let reply = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(1, 0, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(reply.status.code, 200);

    // Cannot create published record.
    let create = alice
        .create_record()
        .protocol(
            definition.protocol.clone(),
            Version::new(1, 0, 0),
            definition.structure.keys().next().unwrap().to_string(),
        )
        .published(true)
        .process()
        .await;
    assert!(create.is_err());

    // Can create unpublished record.
    let create = alice
        .create_record()
        .protocol(
            definition.protocol.clone(),
            Version::new(1, 0, 0),
            definition.structure.keys().next().unwrap().to_string(),
        )
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Alice can query protocol.
    let query = alice
        .query_protocols(ProtocolsFilter {
            protocol: definition.protocol.clone(),
            versions: vec![Version::new(1, 0, 0)],
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    // Bob cannot query protocol.
    let query = bob
        .query_protocols(ProtocolsFilter {
            protocol: definition.protocol.clone(),
            versions: vec![Version::new(1, 0, 0)],
        })
        .target(alice.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert!(query.entries.is_empty());

    // Alice can query record.
    let query = alice
        .query_records(RecordsFilter {
            protocol: Some(definition.protocol.clone()),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);
    assert_eq!(query.entries[0].record_id, create.record_id);

    // Bob cannot query record.
    let query = bob
        .query_records(RecordsFilter {
            protocol: Some(definition.protocol),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert!(query.entries.is_empty());
}
