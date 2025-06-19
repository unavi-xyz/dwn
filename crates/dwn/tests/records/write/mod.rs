use dwn::Dwn;
use dwn_core::message::{Message, descriptor::RecordsWriteBuilder, mime::TEXT_PLAIN};
use tracing_test::traced_test;
use xdid::core::did::Did;

use crate::utils::init_dwn;

mod schema;
mod update;

#[tokio::test]
#[traced_test]
async fn test_write() {
    let (actor, mut dwn) = init_dwn();

    let data = "Hello, world!".as_bytes().to_owned();

    let mut msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();

    expect_success(&actor.did, &mut dwn, msg).await;
}

#[tokio::test]
#[traced_test]
async fn test_require_auth() {
    let (actor, mut dwn) = init_dwn();

    let data = "Hello, world!".as_bytes().to_owned();

    let msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();

    expect_fail(&actor.did, &mut dwn, msg).await;
}

#[tokio::test]
#[traced_test]
async fn test_write_invalid_record_id() {
    let (actor, mut dwn) = init_dwn();

    let data = "Hello, world!".as_bytes().to_owned();

    let mut msg = RecordsWriteBuilder::default()
        .record_id("fake id".to_string())
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();

    expect_fail(&actor.did, &mut dwn, msg).await;
}

async fn expect_success(target: &Did, dwn: &mut Dwn, msg: Message) {
    let record_id = msg.record_id.clone();

    dwn.process_message(target, msg.clone()).await.unwrap();

    let found = dwn
        .record_store
        .read(dwn.data_store.as_ref(), target, &record_id)
        .expect("error reading record")
        .expect("record not found");
    assert_eq!(found.latest_entry, msg);
}

async fn expect_fail(target: &Did, dwn: &mut Dwn, msg: Message) {
    let record_id = msg.record_id.clone();
    assert!(dwn.process_message(target, msg.clone()).await.is_err());
    assert!(
        dwn.record_store
            .read(dwn.data_store.as_ref(), target, &record_id)
            .expect("error reading record")
            .is_none()
    );
}
