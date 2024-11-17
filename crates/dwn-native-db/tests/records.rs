use dwn_core::{message::descriptor::RecordsWriteBuilder, store::RecordStore};
use xdid::methods::key::{p256::P256KeyPair, DidKeyPair, PublicKey};

#[test]
fn test_nativedb_write_read() {
    let did = P256KeyPair::generate().public().to_did();
    let store = dwn_native_db::NativeDbStore::new_in_memory().unwrap();

    let msg = RecordsWriteBuilder::default().build().unwrap();
    store.write(&did, msg.clone()).unwrap();

    let found = store.read(&did, &msg.record_id).unwrap().unwrap();
    assert_eq!(found.initial_entry, msg);
    assert_eq!(found.latest_entry, msg);
}
