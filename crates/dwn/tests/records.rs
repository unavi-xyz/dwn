use dwn::{
    actor::Actor,
    message::{descriptor::records::RecordsFilter, Data},
    store::SurrealStore,
    DWN,
};
use surrealdb::{engine::local::Mem, Surreal};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_records() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = DWN::from(store);

    let actor = Actor::new_did_key(dwn.clone()).unwrap();

    // Create a new record.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    let create = actor
        .create_record()
        .data(data.clone())
        .data_format("text/plain".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    let record_id = create.record_id;

    // Read the record.
    let read = actor
        .read_record(record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Query the record.
    let query = actor
        .query_records(RecordsFilter {
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
    let delete = actor
        .delete_record(record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(delete.reply.status.code, 200);

    // Try to read the deleted record.
    let read = actor
        .read_record(record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, None);

    // Create a record with the same data from another tenant.
    let actor_two = Actor::new_did_key(dwn).unwrap();
    let create_two = actor_two
        .create_record()
        .data(data.clone())
        .data_format("text/plain".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(create_two.reply.status.code, 200);

    // Create a new record.
    let create = actor
        .create_record()
        .data("Hello, world!".bytes().collect::<Vec<_>>())
        .data_format("text/plain".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    let record_id = create.record_id;

    // Read the record.
    let read = actor
        .read_record(record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);

    // Update the record.
    let new_data = "Goodbye, world!".bytes().collect::<Vec<_>>();

    let update = actor
        .update_record(record_id.clone(), record_id.clone())
        .data(new_data.clone())
        .data_format("text/plain".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Read the record.
    let read = actor
        .read_record(record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&new_data)));

    // Update the record again.
    let newer_data = "Hello, again!".bytes().collect::<Vec<_>>();

    let update = actor
        .update_record(record_id.clone(), update.entry_id)
        .data(newer_data.clone())
        .data_format("text/plain".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Read the record.
    let read = actor
        .read_record(record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&newer_data)));

    // Query the record.
    // Only the most recent update message should be returned.
    let query = actor
        .query_records(RecordsFilter {
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
