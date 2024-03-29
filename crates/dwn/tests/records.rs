use std::sync::Arc;

use dwn::{
    actor::Actor,
    message::{descriptor::Filter, Data},
    store::SurrealStore,
    DWN,
};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_records() {
    let store = SurrealStore::new().await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let actor = Actor::new_did_key(dwn).unwrap();

    // Create a new record.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    let create = actor.create().data(data.clone()).process().await.unwrap();
    assert_eq!(create.reply.status.code, 200);

    let record_id = create.record_id;

    // Read the record.
    let read = actor.read(record_id.clone()).process().await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Query the record.
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

    // Delete the record.
    let delete = actor.delete(record_id.clone()).process().await.unwrap();
    assert_eq!(delete.reply.status.code, 200);

    // Query the deleted record.
    let query = actor
        .query(Filter {
            record_id: Some(record_id.clone()),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Try to read the deleted record.
    let read = actor.read(record_id.clone()).process().await;
    assert!(read.is_err());

    // Create a new record.
    let create = actor
        .create()
        .data("Hello, world!".bytes().collect::<Vec<_>>())
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    let record_id = create.record_id;

    // Read the record.
    let read = actor.read(record_id.clone()).process().await.unwrap();
    assert_eq!(read.status.code, 200);

    // Update the record.
    let new_data = "Goodbye, world!".bytes().collect::<Vec<_>>();

    let update = actor
        .update(record_id.clone(), record_id.clone())
        .data(new_data.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Read the record.
    let read = actor.read(record_id.clone()).process().await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&new_data)));

    // Update the record again.
    let newer_data = "Hello, again!".bytes().collect::<Vec<_>>();

    let update = actor
        .update(record_id.clone(), update.entry_id)
        .data(newer_data.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Read the record.
    let read = actor.read(record_id.clone()).process().await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&newer_data)));

    // Query the record.
    // Only the most recent update message should be returned.
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
    assert_eq!(query.entries[0].record_id, record_id);
}
