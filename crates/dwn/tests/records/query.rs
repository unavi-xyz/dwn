use dwn_core::{
    message::descriptor::{DateFilter, DateSort, RecordsQueryBuilder, RecordsWriteBuilder},
    reply::Reply,
};
use tracing_test::traced_test;

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_query_no_filter() {
    let (actor, mut dwn) = init_dwn();

    let msg_1 = RecordsWriteBuilder::default()
        .published(true)
        .build()
        .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_1.clone())
        .unwrap();

    let msg_2 = RecordsWriteBuilder::default()
        .published(true)
        .build()
        .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_2)
        .unwrap();

    let query = RecordsQueryBuilder::default().build().unwrap();

    let reply = match dwn.process_message(&actor.did, query).await.unwrap() {
        Some(Reply::RecordsQuery(v)) => v,
        _ => panic!("invalid reply"),
    };
    assert_eq!(reply.entries.len(), 2);
}

#[tokio::test]
#[traced_test]
async fn test_query_record_id() {
    let (actor, mut dwn) = init_dwn();

    let msg_1 = RecordsWriteBuilder::default()
        .published(true)
        .build()
        .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_1.clone())
        .unwrap();

    let msg_2 = RecordsWriteBuilder::default()
        .published(true)
        .build()
        .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_2)
        .unwrap();

    let query = RecordsQueryBuilder::default()
        .record_id(msg_1.record_id.clone())
        .build()
        .unwrap();

    let reply = match dwn.process_message(&actor.did, query).await.unwrap() {
        Some(Reply::RecordsQuery(v)) => v,
        _ => panic!("invalid reply"),
    };
    assert_eq!(reply.entries.len(), 1);
    assert_eq!(reply.entries[0], msg_1);
}

#[tokio::test]
#[traced_test]
async fn test_query_date_filter() {
    let (actor, mut dwn) = init_dwn();

    let msg_1 = RecordsWriteBuilder::default()
        .published(true)
        .build()
        .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_1.clone())
        .unwrap();

    let msg_2 = RecordsWriteBuilder::default()
        .published(true)
        .build()
        .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_2.clone())
        .unwrap();

    let msg_3 = RecordsWriteBuilder::default()
        .published(true)
        .build()
        .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_3.clone())
        .unwrap();

    let msg_4 = RecordsWriteBuilder::default()
        .published(true)
        .build()
        .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_4.clone())
        .unwrap();

    let query = RecordsQueryBuilder::default()
        .message_timestamp(DateFilter {
            from: *msg_2.descriptor.message_timestamp().unwrap(),
            to: *msg_3.descriptor.message_timestamp().unwrap(),
        })
        .build()
        .unwrap();

    let reply = match dwn.process_message(&actor.did, query).await.unwrap() {
        Some(Reply::RecordsQuery(v)) => v,
        _ => panic!("invalid reply"),
    };
    assert_eq!(reply.entries.len(), 2);
    assert_eq!(reply.entries[0], msg_3);
    assert_eq!(reply.entries[1], msg_2);
}

#[tokio::test]
#[traced_test]
async fn test_query_date_sort() {
    let (actor, mut dwn) = init_dwn();

    let msg_1 = RecordsWriteBuilder::default()
        .published(true)
        .build()
        .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_1.clone())
        .unwrap();

    let msg_2 = RecordsWriteBuilder::default()
        .published(true)
        .build()
        .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_2.clone())
        .unwrap();

    let desc = RecordsQueryBuilder::default()
        .date_sort(DateSort::Descending)
        .build()
        .unwrap();
    let reply = match dwn.process_message(&actor.did, desc).await.unwrap() {
        Some(Reply::RecordsQuery(v)) => v,
        _ => panic!("invalid reply"),
    };
    assert_eq!(reply.entries.len(), 2);
    assert_eq!(reply.entries[0], msg_2);
    assert_eq!(reply.entries[1], msg_1);

    let asc = RecordsQueryBuilder::default()
        .date_sort(DateSort::Ascending)
        .build()
        .unwrap();
    let reply = match dwn.process_message(&actor.did, asc).await.unwrap() {
        Some(Reply::RecordsQuery(v)) => v,
        _ => panic!("invalid reply"),
    };
    assert_eq!(reply.entries.len(), 2);
    assert_eq!(reply.entries[0], msg_1);
    assert_eq!(reply.entries[1], msg_2);
}
