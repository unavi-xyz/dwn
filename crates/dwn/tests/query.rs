use std::sync::Arc;

use dwn::{
    actor::Actor,
    message::descriptor::records::{FilterDateCreated, FilterDateSort, RecordsFilter},
    store::SurrealStore,
    DWN,
};
use surrealdb::{engine::local::Mem, Surreal};
use time::OffsetDateTime;
use tracing_test::traced_test;

const NUM_RECORDS: usize = 12;

fn gen_data(i: usize) -> Vec<u8> {
    format!("Hello from record {}", i)
        .bytes()
        .collect::<Vec<_>>()
}

#[traced_test]
#[tokio::test]
async fn test_filter_date_sort() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));
    let actor = Actor::new_did_key(dwn).unwrap();

    let mut records = Vec::new();

    for i in 0..NUM_RECORDS {
        let data = gen_data(i);
        let create = actor.create_record().data(data).process().await.unwrap();
        records.push(create.record_id.clone());
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
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

#[traced_test]
#[tokio::test]
async fn test_filter_message_timestamp() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));
    let actor = Actor::new_did_key(dwn).unwrap();

    let mut records = Vec::new();

    let start_time = OffsetDateTime::now_utc();
    let mut middle_time = OffsetDateTime::now_utc();
    let mut two_thirds_time = OffsetDateTime::now_utc();

    for i in 0..NUM_RECORDS {
        if i == NUM_RECORDS / 2 {
            middle_time = OffsetDateTime::now_utc();
        }

        if i == NUM_RECORDS / 3 * 2 {
            two_thirds_time = OffsetDateTime::now_utc();
        }

        let data = gen_data(i);
        let create = actor.create_record().data(data).process().await.unwrap();
        records.push(create.record_id.clone());

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let end_time = OffsetDateTime::now_utc();

    // From start to end time.
    let query = actor
        .query_records(RecordsFilter {
            date_sort: Some(FilterDateSort::CreatedAscending),
            message_timestamp: Some(FilterDateCreated {
                from: Some(start_time),
                to: Some(end_time),
            }),
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

    // From middle time.
    let query = actor
        .query_records(RecordsFilter {
            date_sort: Some(FilterDateSort::CreatedAscending),
            message_timestamp: Some(FilterDateCreated {
                from: Some(middle_time),
                to: None,
            }),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), NUM_RECORDS / 2);

    let base = NUM_RECORDS / 2;
    for (i, record) in query.entries.iter().enumerate() {
        assert_eq!(record.record_id, records[base + i]);
    }

    // To middle time.
    let query = actor
        .query_records(RecordsFilter {
            date_sort: Some(FilterDateSort::CreatedAscending),
            message_timestamp: Some(FilterDateCreated {
                from: None,
                to: Some(middle_time),
            }),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), NUM_RECORDS / 2);

    for (i, record) in query.entries.iter().enumerate() {
        assert_eq!(record.record_id, records[i]);
    }

    // From middle to end time.
    let query = actor
        .query_records(RecordsFilter {
            date_sort: Some(FilterDateSort::CreatedAscending),
            message_timestamp: Some(FilterDateCreated {
                from: Some(middle_time),
                to: Some(end_time),
            }),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), NUM_RECORDS / 2);

    let base = NUM_RECORDS / 2;
    for (i, record) in query.entries.iter().enumerate() {
        assert_eq!(record.record_id, records[base + i]);
    }

    // From middle to two-thirds time.
    let query = actor
        .query_records(RecordsFilter {
            date_sort: Some(FilterDateSort::CreatedAscending),
            message_timestamp: Some(FilterDateCreated {
                from: Some(middle_time),
                to: Some(two_thirds_time),
            }),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), (NUM_RECORDS / 2) - (NUM_RECORDS / 3));

    let base = NUM_RECORDS / 2;
    for (i, record) in query.entries.iter().enumerate() {
        assert_eq!(record.record_id, records[base + i]);
    }
}

#[traced_test]
#[tokio::test]
async fn test_query_records_delete() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));
    let actor = Actor::new_did_key(dwn).unwrap();

    // Create a new record.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();
    let create = actor.create_record().data(data).process().await.unwrap();

    let record_id = create.record_id.clone();

    // Delete the record.
    let delete = actor
        .delete_record(record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(delete.reply.status.code, 200);

    // Query the RecordsDelete message.
    let query = actor
        .query_records(RecordsFilter {
            ..Default::default()
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let entry = &query.entries[0];
    assert_eq!(entry.record_id, record_id);
}
