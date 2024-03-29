use std::sync::Arc;

use dwn::{
    actor::{Actor, MessageBuilder},
    message::{data::Data, descriptor::Filter},
    store::SurrealStore,
    DWN,
};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_publish() {
    let store = SurrealStore::new().await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let actor = Actor::new_did_key(dwn.clone()).unwrap();

    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    // Create an unpublished record.
    let create = actor.create().data(data.clone()).process().await.unwrap();
    let record_id = create.record_id;

    // The record can only be read with authorization.
    let read = actor.read(record_id.clone()).process().await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    let read = actor
        .read(record_id.clone())
        .authorized(false)
        .process()
        .await;
    assert!(read.is_err());

    // The record can only be queried with authorization.
    let query = actor
        .query(Filter {
            record_id: Some(record_id.clone()),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let query = actor
        .query(Filter {
            record_id: Some(record_id.clone()),
            ..Default::default()
        })
        .authorized(false)
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Create a published record.
    let create = actor
        .create()
        .data(data.clone())
        .published(true)
        .process()
        .await
        .unwrap();

    let record_id = create.record_id;

    // Can read with or without authorization.
    let read = actor.read(record_id.clone()).process().await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    let read = actor
        .read(record_id.clone())
        .authorized(false)
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Can query with or without authorization.
    let query = actor
        .query(Filter {
            record_id: Some(record_id.clone()),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);
    assert_eq!(query.entries[0].record_id, record_id.clone());

    let query = actor
        .query(Filter {
            record_id: Some(record_id.clone()),
            ..Default::default()
        })
        .authorized(false)
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);
    assert_eq!(query.entries[0].record_id, record_id.clone());
}
