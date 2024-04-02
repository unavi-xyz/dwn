use std::sync::Arc;

use dwn::{
    actor::Actor,
    message::descriptor::records::{FilterDateSort, RecordsFilter},
    store::SurrealStore,
    DWN,
};
use tracing_test::traced_test;

const NUM_RECORDS: usize = 5;

fn gen_data(i: usize) -> Vec<u8> {
    format!("Hello from record {}", i)
        .bytes()
        .collect::<Vec<_>>()
}

#[traced_test]
#[tokio::test]
async fn test_date_sort() {
    let store = SurrealStore::new().await.unwrap();
    let dwn = Arc::new(DWN::from(store));
    let actor = Actor::new_did_key(dwn).unwrap();

    let mut records = Vec::new();

    for i in 0..NUM_RECORDS {
        let data = gen_data(i);
        let create = actor.create_record().data(data).process().await.unwrap();
        records.push(create.record_id.clone());
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // CreatedAscending
    let query = actor
        .query_records(RecordsFilter {
            date_sort: Some(FilterDateSort::CreatedAscending),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), NUM_RECORDS);

    for (i, record) in query.entries.iter().enumerate() {
        assert_eq!(record.record_id, records[i]);
    }

    // CreatedDescending
    let query = actor
        .query_records(RecordsFilter {
            date_sort: Some(FilterDateSort::CreatedDescending),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), NUM_RECORDS);

    for (i, record) in query.entries.iter().enumerate() {
        assert_eq!(record.record_id, records[NUM_RECORDS - i - 1]);
    }

    // PublishedAscending
    let query = actor
        .query_records(RecordsFilter {
            date_sort: Some(FilterDateSort::PublishedAscending),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), NUM_RECORDS);

    for (i, record) in query.entries.iter().enumerate() {
        assert_eq!(record.record_id, records[i]);
    }

    // PublishedDescending
    let query = actor
        .query_records(RecordsFilter {
            date_sort: Some(FilterDateSort::PublishedDescending),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), NUM_RECORDS);

    for (i, record) in query.entries.iter().enumerate() {
        assert_eq!(record.record_id, records[NUM_RECORDS - i - 1]);
    }
}
