use dwn::{builders::records::write::RecordsWriteBuilder, DWN};
use dwn_core::message::mime::TEXT_PLAIN;
use dwn_native_db::NativeDbStore;
use tracing_test::traced_test;

#[test]
#[traced_test]
fn test_write_data() {
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = DWN::from(store);

    let target = "did:example:123".to_string();
    let data = "hello, world!".as_bytes().to_owned();

    let msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();

    let record_id = msg.record_id.clone();

    dwn.process_message(&target, msg.clone()).unwrap();

    let found = dwn
        .record_store
        .read(&target, &record_id)
        .expect("error reading record")
        .expect("record not found");
    assert_eq!(found, msg);
}
