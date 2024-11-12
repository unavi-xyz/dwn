use dwn::builders::records::{RecordsReadBuilder, RecordsWriteBuilder};
use dwn_core::{
    message::mime::TEXT_PLAIN,
    reply::{RecordsReadReply, Reply},
};
use tracing_test::traced_test;

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_read_published() {
    let (actor, mut dwn) = init_dwn();

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
        Some(Reply::RecordsRead(m)) => m,
        _ => panic!("invalid reply"),
    };
    assert_eq!(reply.entry, Some(write));
}

#[tokio::test]
#[traced_test]
async fn test_read_unpublished() {
    let (actor, mut dwn) = init_dwn();

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
        Some(Reply::RecordsRead(m)) => m,
        _ => panic!("invalid reply"),
    };
    assert_eq!(reply.entry, Some(write));
}

#[tokio::test]
#[traced_test]
async fn test_read_unpublished_requires_auth() {
    let (actor, mut dwn) = init_dwn();

    let data = "hello, world!".as_bytes().to_owned();

    let write = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();
    dwn.record_store.write(&actor.did, write.clone()).unwrap();

    let read = RecordsReadBuilder::new(write.record_id.clone())
        .build()
        .unwrap();
    assert_eq!(
        dwn.process_message(&actor.did, read).await.unwrap(),
        Some(Reply::RecordsRead(Box::new(RecordsReadReply {
            entry: None
        })))
    )
}
