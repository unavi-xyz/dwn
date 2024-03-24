use dwn::{
    actor::{Actor, CreateRecord},
    message::descriptor::Filter,
    store::SurrealDB,
    DWN,
};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_records() {
    let db = SurrealDB::new().await.unwrap();
    let dwn = DWN::new(db);

    let actor = Actor::new_did_key(dwn).unwrap();

    // Create a new record.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    let create = actor
        .create(CreateRecord {
            data: Some(data.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    let record_id = create.record_id;

    // Read the record.
    let read = actor.read(record_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.data, Some(data.clone()));

    // Query the record.
    let query = actor
        .query(Filter {
            record_id: Some(record_id.clone()),
            ..Default::default()
        })
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
        .query(Filter {
            record_id: Some(record_id.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 0);

    // Try to read the deleted record.
    let read = actor.read(record_id.clone()).await;
    assert!(read.is_err());

    // Create a new record.
    let create = actor
        .create(CreateRecord {
            data: Some(data.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    let record_id = create.record_id;

    // Read the record.
    let read = actor.read(record_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);

    // Update the record.
    let new_data = "Goodbye, world!".bytes().collect::<Vec<_>>();

    let update = actor
        .update(
            record_id.clone(),
            record_id.clone(),
            CreateRecord {
                data: Some(new_data.clone()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(update.status.code, 200);

    // Read the record.
    let reply = actor.read(record_id.clone()).await.unwrap();
    assert_eq!(reply.status.code, 200);
    assert_eq!(reply.data, Some(new_data));

    // Query the record.
    // Only the update message should be returned.
    let query = actor
        .query(Filter {
            record_id: Some(record_id.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);
    assert_eq!(query.entries[0].record_id, record_id);
}
