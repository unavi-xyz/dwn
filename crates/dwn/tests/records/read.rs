use dwn::{
    builders::records::{read::RecordsReadBuilder, write::RecordsWriteBuilder},
    Dwn, Reply,
};
use dwn_core::message::mime::TEXT_PLAIN;
use dwn_native_db::NativeDbStore;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_read_data() {
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);

    let target = "did:example:123";
    let data = "hello, world!".as_bytes().to_owned();

    let msg_write = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();
    dwn.record_store.write(target, msg_write.clone()).unwrap();

    let msg_read = RecordsReadBuilder::new(msg_write.record_id.clone())
        .build()
        .unwrap();
    let reply = match dwn.process_message(target, msg_read).await.unwrap() {
        Reply::RecordsRead(m) => m,
        _ => panic!("invalid reply"),
    };
    assert_eq!(*reply, msg_write);
}
