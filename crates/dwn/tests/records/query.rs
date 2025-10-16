use dwn_core::{
    message::descriptor::{
        DateFilter, DateSort, RecordFilter, RecordsQueryBuilder, RecordsWriteBuilder,
    },
    reply::Reply,
};
use tracing_test::traced_test;

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_query_no_filter() {
    let (actor, dwn) = init_dwn();

    let msg_1 = RecordsWriteBuilder {
        published: Some(true),
        ..Default::default()
    }
    .build()
    .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_1.clone())
        .unwrap();

    let msg_2 = RecordsWriteBuilder {
        published: Some(true),
        ..Default::default()
    }
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
    let (actor, dwn) = init_dwn();

    let msg_1 = RecordsWriteBuilder {
        published: Some(true),
        ..Default::default()
    }
    .build()
    .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_1.clone())
        .unwrap();

    let msg_2 = RecordsWriteBuilder {
        published: Some(true),
        ..Default::default()
    }
    .build()
    .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_2)
        .unwrap();

    let query = RecordsQueryBuilder {
        filter: RecordFilter {
            record_id: Some(msg_1.record_id.clone()),
            ..Default::default()
        },
    }
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

    let msg_1 = RecordsWriteBuilder {
        published: Some(true),
        ..Default::default()
    }
    .build()
    .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_1.clone())
        .unwrap();

    let msg_2 = RecordsWriteBuilder {
        published: Some(true),
        ..Default::default()
    }
    .build()
    .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_2.clone())
        .unwrap();

    let msg_3 = RecordsWriteBuilder {
        published: Some(true),
        ..Default::default()
    }
    .build()
    .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_3.clone())
        .unwrap();

    let msg_4 = RecordsWriteBuilder {
        published: Some(true),
        ..Default::default()
    }
    .build()
    .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_4.clone())
        .unwrap();

    let query = RecordsQueryBuilder {
        filter: RecordFilter {
            date_created: Some(DateFilter {
                from: *msg_2.descriptor.message_timestamp().unwrap(),
                to: *msg_3.descriptor.message_timestamp().unwrap(),
            }),
            ..Default::default()
        },
    }
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

    let msg_1 = RecordsWriteBuilder {
        published: Some(true),
        ..Default::default()
    }
    .build()
    .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_1.clone())
        .unwrap();

    let msg_2 = RecordsWriteBuilder {
        published: Some(true),
        ..Default::default()
    }
    .build()
    .unwrap();
    dwn.record_store
        .write(dwn.data_store.as_ref(), &actor.did, msg_2.clone())
        .unwrap();

    let desc = RecordsQueryBuilder {
        filter: RecordFilter {
            date_sort: Some(DateSort::Descending),
            ..Default::default()
        },
    }
    .build()
    .unwrap();
    let reply = match dwn.process_message(&actor.did, desc).await.unwrap() {
        Some(Reply::RecordsQuery(v)) => v,
        _ => panic!("invalid reply"),
    };
    assert_eq!(reply.entries.len(), 2);
    assert_eq!(reply.entries[0], msg_2);
    assert_eq!(reply.entries[1], msg_1);

    let asc = RecordsQueryBuilder {
        filter: RecordFilter {
            date_sort: Some(DateSort::Ascending),
            ..Default::default()
        },
    }
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
