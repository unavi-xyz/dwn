use std::sync::Arc;

use dwn::{
    actor::{records::Encryption, Actor},
    message::Data,
    store::SurrealStore,
    DWN,
};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_encrypt() {
    let store = SurrealStore::new().await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let actor = Actor::new_did_key(dwn).unwrap();

    // Create an encrypted record.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();
    let encryption = Encryption::generate_aes256().unwrap();

    let create = actor
        .create_record()
        .data(data.clone())
        .encryption(&encryption)
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Read the record.
    let read = actor.read_record(create.record_id).process().await.unwrap();
    assert_eq!(read.status.code, 200);

    // Decrypt the data.
    let encrypted = match read.record.data.unwrap() {
        Data::Encrypted(encrypted) => encrypted,
        _ => panic!("expected encrypted data"),
    };
    let decrypted = encrypted.decrypt(encryption).unwrap();
    assert_eq!(decrypted, data);
}
