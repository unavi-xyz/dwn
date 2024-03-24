use dwn::{
    actor::{Actor, CreateRecord},
    message::descriptor::Filter,
    store::SurrealDB,
    DWN,
};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_publish() {
    let db = SurrealDB::new().await.unwrap();
    let dwn = DWN::new(db);

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn).unwrap();

    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    // Alice creates an unpublished record.
    let create = alice
        .create(CreateRecord {
            data: Some(data.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    let record_id = create.record_id;

    // Alice can read the record.
    let read = alice.read(record_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.data, Some(data.clone()));

    // Bob cannot read the record.
    let read = bob.read(record_id.clone()).await;
    assert!(read.is_err());

    // Bob cannot query the record.
    let query = bob
        .query(Filter {
            record_id: Some(record_id.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Alice creates a published record.
    let create = alice
        .create(CreateRecord {
            data: Some(data.clone()),
            published: true,
            ..Default::default()
        })
        .await
        .unwrap();

    let record_id = create.record_id;

    // Alice can read the record.
    let read = alice.read(record_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.data, Some(data.clone()));

    // Bob can read the record.
    let read = bob.read(record_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.data, Some(data.clone()));

    // Bob can query the record.
    let query = bob
        .query(Filter {
            record_id: Some(record_id.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);
    assert_eq!(query.entries[0].record_id, record_id.clone());
}
