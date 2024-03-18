use dwn::{message::data::EncryptedData, store::SurrealDB, Actor, DWN};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_encrypt() {
    let db = SurrealDB::new().await.unwrap();
    let dwn = DWN {
        data_store: db.clone(),
        message_store: db,
    };

    let actor = Actor::new_did_key(dwn.clone()).unwrap();

    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    // Create an encrypted record.
    let write = actor
        .write()
        .data(data.clone())
        .encrypt(true)
        .send()
        .await
        .unwrap();
    assert_eq!(write.reply.status.code, 200);

    // Read the record.
    let read = actor.read(write.entry_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_ne!(read.data, Some(data.clone()));

    // Decrypt the data.
    let encrypted = serde_json::from_slice::<EncryptedData>(&read.data.unwrap()).unwrap();
    let decrypted = encrypted.decrypt(&write.encryption_key.unwrap()).unwrap();
    assert_eq!(decrypted, data);
}
