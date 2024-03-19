use dwn::{store::SurrealDB, Actor, DWN};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_publish() {
    let db = SurrealDB::new().await.unwrap();
    let dwn = DWN {
        data_store: db.clone(),
        message_store: db,
    };

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn).unwrap();

    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    // Alice creates an unpublished record.
    let write = alice.write().data(data.clone()).send().await.unwrap();

    // Alice can read the record.
    let read = alice.read(write.entry_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.data, Some(data.clone()));

    // Bob cannot read the record.
    let read = bob.read(write.entry_id.clone()).await;
    assert!(read.is_err());

    // Bob cannot query the record.
    let query = bob
        .query()
        .record_id(write.entry_id.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Alice creates a published record.
    let write = alice
        .write()
        .data(data.clone())
        .published(true)
        .send()
        .await
        .unwrap();

    // Alice can read the record.
    let read = alice.read(write.entry_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.data, Some(data.clone()));

    // Bob can read the record.
    let read = bob.read(write.entry_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.data, Some(data.clone()));

    // Bob can query the record.
    let query = bob
        .query()
        .record_id(write.entry_id.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);
    assert_eq!(query.entries[0].record_id, write.entry_id.clone());
}
