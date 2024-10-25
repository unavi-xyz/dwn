use dwn::{builders::records::write::RecordsWriteBuilder, Dwn};
use dwn_core::message::{mime::TEXT_PLAIN, Message};
use tracing_test::traced_test;
use xdid::core::did::Did;

use crate::utils::init_dwn;

mod schema;

#[tokio::test]
#[traced_test]
async fn test_write_data() {
    let (actor, dwn) = init_dwn();

    let data = "hello, world!".as_bytes().to_owned();

    let mut msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();

    expect_success(&actor.did, &dwn, msg).await;
}

#[tokio::test]
#[traced_test]
async fn test_require_auth() {
    let (actor, dwn) = init_dwn();

    let data = "hello, world!".as_bytes().to_owned();

    let msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();

    expect_fail(&actor.did, &dwn, msg).await;
}

async fn expect_success(target: &Did, dwn: &Dwn, msg: Message) {
    let record_id = msg.record_id.clone();

    dwn.process_message(target, msg.clone()).await.unwrap();

    let found = dwn
        .record_store
        .read(target, &record_id)
        .expect("error reading record")
        .expect("record not found");
    assert_eq!(found, msg);
}

async fn expect_fail(target: &Did, dwn: &Dwn, msg: Message) {
    let record_id = msg.record_id.clone();
    assert!(dwn.process_message(target, msg.clone()).await.is_err());
    assert!(dwn
        .record_store
        .read(target, &record_id)
        .expect("error reading record")
        .is_none());
}
