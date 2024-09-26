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
pub async fn test_recipient_of_read() {
    let definition: ProtocolDefinition = serde_json::from_str(DEFINITION).unwrap();

    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = DWN::from(store);

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn.clone()).unwrap();
    let charlie = Actor::new_did_key(dwn).unwrap();

    let reply = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(1, 0, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(reply.status.code, 200);

    // Bob creates post.
    let post = bob
        .create_record()
        .protocol(
            definition.protocol.clone(),
            Version::new(1, 0, 0),
            "post".to_string(),
        )
        .published(true)
        .target(alice.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(post.reply.status.code, 200);

    // Bob writes reply.
    let create = bob
        .create_record()
        .protocol(
            definition.protocol.clone(),
            Version::new(1, 0, 0),
            "post/reply".to_string(),
        )
        .parent_context_id(post.record_id)
        .published(true)
        .target(alice.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Bob can read reply.
    let read = bob
        .read_record(create.record_id.clone())
        .target(alice.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.record_id, create.record_id);

    // Alice can read reply.
    let read = alice
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.record_id, create.record_id);

    // Charlie cannot read reply.
    let read = charlie
        .read_record(create.record_id.clone())
        .target(alice.did.clone())
        .process()
        .await;
    assert!(read.is_err());
}
