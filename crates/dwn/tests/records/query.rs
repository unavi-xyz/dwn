use dwn::builders::records::{query::RecordsQueryBuilder, write::RecordsWriteBuilder};
use dwn_core::{
    message::{mime::TEXT_PLAIN, DateFilter, DateSort},
    reply::Reply,
};
use tracing_test::traced_test;

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_query_no_filter() {
    let (actor, dwn) = init_dwn();

    let msg_1 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "1".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, msg_1.clone()).unwrap();

    let msg_2 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "2".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, msg_2).unwrap();

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
    let (actor, dwn) = init_dwn();

    let msg_1 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "1".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, msg_1.clone()).unwrap();

    let msg_2 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "2".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, msg_2).unwrap();

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
    let (actor, dwn) = init_dwn();

    let msg_1 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "1".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, msg_1.clone()).unwrap();

    let msg_2 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "2".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, msg_2.clone()).unwrap();

    let msg_3 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "3".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, msg_3.clone()).unwrap();

    let msg_4 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "4".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, msg_4.clone()).unwrap();

    let query = RecordsQueryBuilder::default()
        .message_timestamp(DateFilter {
            from: msg_2.descriptor.message_timestamp,
            to: msg_3.descriptor.message_timestamp,
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
    let (actor, dwn) = init_dwn();

    let msg_1 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "1".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, msg_1.clone()).unwrap();

    let msg_2 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "2".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, msg_2.clone()).unwrap();

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
