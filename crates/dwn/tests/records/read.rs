use dwn::{
    builders::records::{read::RecordsReadBuilder, write::RecordsWriteBuilder},
    Reply,
};
use dwn_core::message::mime::TEXT_PLAIN;
use tracing_test::traced_test;

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_read_published() {
    let (actor, dwn) = init_dwn();

    let data = "hello, world!".as_bytes().to_owned();

    let write = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .published(true)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, write.clone()).unwrap();

    let read = RecordsReadBuilder::new(write.record_id.clone())
        .build()
        .unwrap();
    let reply = match dwn.process_message(&actor.did, read).await.unwrap() {
        Reply::RecordsRead(m) => m,
        _ => panic!("invalid reply"),
    };
    assert_eq!(*reply, write);
}

#[tokio::test]
#[traced_test]
async fn test_read_unpublished() {
    let (actor, dwn) = init_dwn();

    let data = "hello, world!".as_bytes().to_owned();

    let write = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, write.clone()).unwrap();

    let mut read = RecordsReadBuilder::new(write.record_id.clone())
        .build()
        .unwrap();
    actor.authorize(&mut read).unwrap();

    let reply = match dwn.process_message(&actor.did, read).await.unwrap() {
        Reply::RecordsRead(m) => m,
        _ => panic!("invalid reply"),
    };
    assert_eq!(*reply, write);
}

#[tokio::test]
#[traced_test]
async fn test_read_unpublished_requires_auth() {
    let (actor, dwn) = init_dwn();

    let data = "hello, world!".as_bytes().to_owned();

    let write = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, write.clone()).unwrap();

    let read = RecordsReadBuilder::new(write.record_id.clone())
        .build()
        .unwrap();
    assert!(dwn.process_message(&actor.did, read).await.is_err())
}
