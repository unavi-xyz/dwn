use std::sync::Arc;

use dwn::{
    actor::Actor, message::descriptor::protocols::ProtocolDefinition, store::SurrealStore, DWN,
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

    let actor = Actor::new_did_key(dwn.clone()).unwrap();

    let reply = actor
        .register_protocol(definition.clone())
        .protocol_version(Version::new(1, 0, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(reply.status.code, 200);

    // Cannot create published record.
    let create = actor
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
    let create = actor
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
}
