use dwn::{
    actor::{Actor, MessageBuilder},
    message::{descriptor::records::RecordsFilter, Data},
    store::SurrealStore,
    DWN,
};
use surrealdb::{engine::local::Mem, Surreal};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_publish() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = DWN::from(store);

    let actor = Actor::new_did_key(dwn).unwrap();

    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    // Create an unpublished record.
    let create = actor
        .create_record()
        .data(data.clone())
        .data_format("application/json".to_string())
        .process()
        .await
        .unwrap();
    let record_id = create.record_id;

    // The record can only be read with authorization.
    let read = actor
        .read_record(record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    let read = actor
        .read_record(record_id.clone())
        .authorized(false)
        .process()
        .await;
    assert!(read.is_err());

    // The record can only be queried with authorization.
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

    let query = actor
        .query_records(RecordsFilter {
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
        .create_record()
        .data(data.clone())
        .data_format("application/json".to_string())
        .published(true)
        .process()
        .await
        .unwrap();

    let record_id = create.record_id;

    // Can read with or without authorization.
    let read = actor
        .read_record(record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    let read = actor
        .read_record(record_id.clone())
        .authorized(false)
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Can query with or without authorization.
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

    let query = actor
        .query_records(RecordsFilter {
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
