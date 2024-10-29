use dwn::{
    builders::records::{query::RecordsQueryBuilder, write::RecordsWriteBuilder},
    Reply,
};
use dwn_core::message::mime::TEXT_PLAIN;
use tracing_test::traced_test;

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_query_record_id() {
    let (actor, dwn) = init_dwn();

    let data = "hello, world!".as_bytes().to_owned();

    let write = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, write.clone()).unwrap();

    let query = RecordsQueryBuilder::default()
        .record_id(write.record_id.clone())
        .build()
        .unwrap();

    let reply = match dwn.process_message(&actor.did, query).await.unwrap() {
        Reply::RecordsQuery(m) => m,
        _ => panic!("invalid reply"),
    };
    assert_eq!(reply.len(), 1);
    assert_eq!(reply[0], write);
}
