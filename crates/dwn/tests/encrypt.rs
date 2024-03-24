use dwn::{
    actor::{Actor, CreateRecord, Encryption},
    message::data::EncryptedData,
    store::SurrealStore,
    DWN,
};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_encrypt() {
    let store = SurrealStore::new().await.unwrap();
    let dwn = DWN::new(store);

    let actor = Actor::new_did_key(dwn).unwrap();

    // Create an encrypted record.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();
    let encryption = Encryption::generate_aes256().unwrap();

    let create = actor
        .create(CreateRecord {
            data: Some(data.clone()),
            encryption: Some(&encryption),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Read the record.
    let read = actor.read(create.record_id).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_ne!(read.data, Some(data.clone()));

    // Decrypt the data.
    let encrypted = serde_json::from_slice::<EncryptedData>(&read.data.unwrap()).unwrap();
    let decrypted = encrypted.decrypt(encryption).unwrap();
    assert_eq!(decrypted, data);
}
