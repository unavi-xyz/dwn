use dwn::{builders::records::write::RecordsWriteBuilder, Dwn};
use dwn_core::message::{mime::TEXT_PLAIN, Message};
use dwn_native_db::NativeDbStore;
use tracing_test::traced_test;

mod schema;

#[tokio::test]
#[traced_test]
async fn test_write_data() {
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);

    let target = "did:example:123";
    let data = "hello, world!".as_bytes().to_owned();

    let msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();

    expect_success(&dwn, msg, target).await;
}

async fn expect_success(dwn: &Dwn, msg: Message, target: &str) {
    let record_id = msg.record_id.clone();

    dwn.process_message(target, msg.clone()).await.unwrap();

    let found = dwn
        .record_store
        .read(target, &record_id)
        .expect("error reading record")
        .expect("record not found");
    assert_eq!(found, msg);
}

async fn expect_fail(dwn: &Dwn, msg: Message, target: &str) {
    let record_id = msg.record_id.clone();
    assert!(dwn.process_message(target, msg.clone()).await.is_err());
    assert!(dwn
        .record_store
        .read(target, &record_id)
        .expect("error reading record")
        .is_none());
}
