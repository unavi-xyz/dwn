use dwn::{
    actor::{records::Encryption, Actor},
    message::Data,
    store::SurrealStore,
    DWN,
};
use surrealdb::{engine::local::Mem, Surreal};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_encrypt() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = DWN::from(store);

    let actor = Actor::new_did_key(dwn).unwrap();

    // Create an encrypted record.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();
    let encryption = Encryption::generate_aes256();

    let create = actor
        .create_record()
        .data(data.clone())
        .data_format("text/plain".to_string())
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
