use dwn::{store::SurrealDB, Actor, DWN};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_records() {
    let db = SurrealDB::new().await.unwrap();
    let dwn = DWN {
        data_store: db.clone(),
        message_store: db,
    };

    let actor = Actor::new_did_key(dwn).unwrap();

    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    // Create new record.
    let write = actor.write().data(data.clone()).send().await.unwrap();
    let record_id = write.entry_id.clone();
    assert_eq!(write.reply.status.code, 200);

    // Read the record.
    let read = actor.read(record_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.data, Some(data.clone()));

    // Query the record.
    let query = actor
        .query()
        .record_id(record_id.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);
    assert_eq!(query.entries[0].record_id, record_id.clone());

    // Delete the record.
    let delete = actor.delete(record_id.clone()).await.unwrap();
    assert_eq!(delete.status.code, 200);

    // Query the deleted record.
    let query = actor
        .query()
        .record_id(record_id.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Try to read the deleted record.
    let read = actor.read(record_id.clone()).await;
    assert!(read.is_err());

    // Create a new record.
    let write = actor.write().data(data.clone()).send().await.unwrap();
    let record_id = write.entry_id.clone();

    // Read the record.
    let read = actor.read(record_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);

    // Update the record.
    let new_data = "Goodbye, world!".bytes().collect::<Vec<_>>();
    let update = actor
        .write()
        .data(new_data.clone())
        .parent_id(record_id.clone())
        .record_id(record_id.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Read the record.
    let reply = actor.read(record_id.clone()).await.unwrap();
    assert_eq!(reply.status.code, 200);
    assert_eq!(reply.data, Some(new_data));

    // Query the record.
    // Only the update message should be returned.
    let query = actor
        .query()
        .record_id(record_id.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);
    assert_eq!(query.entries[0].record_id, record_id);
}
