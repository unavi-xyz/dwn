use dwn::{
    actor::{Actor, MessageBuilder},
    message::descriptor::protocols::ProtocolDefinition,
    store::SurrealStore,
    DWN,
};
use semver::Version;
use surrealdb::{engine::local::Mem, Surreal};

const DEFINITION: &str = include_str!("./protocol.json");

#[tokio::test]
pub async fn test_anyone_read_write() {
    let definition: ProtocolDefinition = serde_json::from_str(DEFINITION).unwrap();

    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = DWN::from(store);

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn).unwrap();

    let reply = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(1, 0, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(reply.status.code, 200);

    // Alice can write.
    let create = alice
        .create_record()
        .published(true)
        .protocol(
            definition.protocol.clone(),
            Version::new(1, 0, 0),
            definition.structure.keys().next().unwrap().to_string(),
        )
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Alice can read.
    let read = alice
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.record_id, create.record_id);

    // Bob can read Alice.
    let read = bob
        .read_record(create.record_id.clone())
        .target(alice.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.record_id, create.record_id);

    // Bob can write.
    let create = bob
        .create_record()
        .published(true)
        .protocol(
            definition.protocol.clone(),
            Version::new(1, 0, 0),
            definition.structure.keys().next().unwrap().to_string(),
        )
        .target(alice.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Bob can read.
    let read = bob
        .read_record(create.record_id.clone())
        .target(alice.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.record_id, create.record_id);

    // Alice can read Bob.
    let read = alice
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.record_id, create.record_id);
}
