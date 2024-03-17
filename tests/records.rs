use dwn::{store::SurrealDB, Actor, DWN};

#[tokio::test]
async fn test_records() {
    let db = SurrealDB::new().await.unwrap();
    let dwn = DWN {
        data_store: db.clone(),
        message_store: db,
    };

    let actor = Actor::new_did_key(dwn).unwrap();

    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    // Create new record.
    let res = actor.write().data(data.clone()).send().await.unwrap();
    let record_id = res.entry_id.clone();
    assert_eq!(res.reply.status.code, 200);

    // Read the record.
    let reply = actor.read(record_id.clone()).await.unwrap();
    assert_eq!(reply.status.code, 200);
    assert_eq!(reply.data, Some(data.clone()));

    // Query the record.
    let query = actor.query().send().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);
    assert_eq!(query.entries[0].record_id, record_id.clone());

    // Update the record.
    let new_data = "Goodbye, world!".bytes().collect::<Vec<_>>();
    let update = actor
        .write()
        .data(new_data.clone())
        .parent_id(record_id.clone())
        .record_id(record_id.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Read the updated record.
    let reply = actor.read(record_id.clone()).await.unwrap();
    assert_eq!(reply.status.code, 200);
    assert_eq!(reply.data, Some(new_data));
}
