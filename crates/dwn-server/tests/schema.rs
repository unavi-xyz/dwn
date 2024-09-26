use std::sync::Arc;

use axum::{routing::get, Json, Router};
use dwn::{
    actor::{records::Encryption, Actor},
    message::Data,
    store::SurrealStore,
    DWN,
};
use serde_json::json;
use surrealdb::{engine::local::Mem, Surreal};
use tokio::net::TcpListener;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_records_schema() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = DWN::from(store);

    let actor = Actor::new_did_key(dwn).unwrap();

    // Host a JSON schema over HTTP.
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "data": {
                "type": "string"
            }
        },
        "required": ["data"]
    });

    let port = port_scanner::request_open_port().unwrap();
    let schema_url = format!("http://localhost:{}/schema.json", port);

    tokio::spawn(async move {
        let schema = Arc::new(schema);
        let router = Router::new().route("/schema.json", get(|| async { Json(schema) }));

        let url = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(url).await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });

    // Wait for the server to start.
    while port_scanner::scan_port(port) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Data must follow the schema.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    let create = actor
        .create_record()
        .data(data.clone())
        .data_format("application/json".to_string())
        .schema(schema_url.clone())
        .process()
        .await;
    assert!(create.is_err());

    let data = r#"{"data": "Hello, world!"}"#.bytes().collect::<Vec<_>>();

    let create = actor
        .create_record()
        .data(data.clone())
        .data_format("application/json".to_string())
        .schema(schema_url.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Must specify data format.
    let create_wrong = actor
        .create_record()
        .data(data.clone())
        .schema(schema_url.clone())
        .process()
        .await;
    assert!(create_wrong.is_err());

    // Multiple records can use the same schema.
    let create_two = actor
        .create_record()
        .data(data.clone())
        .data_format("application/json".to_string())
        .schema(schema_url.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(create_two.reply.status.code, 200);
    assert!(create_two.record_id != create.record_id);

    // Data must follow the schema when updating.
    let data = r#"{"wrong_key": "Hello again!"}"#.bytes().collect::<Vec<_>>();

    let update = actor
        .update_record(create.record_id.clone(), create.entry_id.clone())
        .data(data.clone())
        .data_format("application/json".to_string())
        .process()
        .await;
    assert!(update.is_err());

    let update = actor
        .update_record(create.record_id.clone(), create.entry_id.clone())
        .data(data.clone())
        .data_format("application/json".to_string())
        .schema(schema_url.clone())
        .process()
        .await;
    assert!(update.is_err());

    let data = r#"{"data": "Hello again!"}"#.bytes().collect::<Vec<_>>();

    let update = actor
        .update_record(create.record_id.clone(), create.entry_id.clone())
        .data(data.clone())
        .data_format("application/json".to_string())
        .process()
        .await;
    assert!(update.is_err());

    let update = actor
        .update_record(create.record_id.clone(), create.entry_id.clone())
        .data(data.clone())
        .data_format("application/json".to_string())
        .schema("http://localhost:1234/new-schema.json".to_string())
        .process()
        .await;
    assert!(update.is_err());

    // Data cannot be encrypted
    let encryption = Encryption::generate_aes256();
    let update = actor
        .update_record(create.record_id.clone(), create.entry_id.clone())
        .data(data.clone())
        .data_format("application/json".to_string())
        .schema(schema_url.clone())
        .encryption(&encryption)
        .process()
        .await;
    assert!(update.is_err());

    let update = actor
        .update_record(create.record_id.clone(), create.entry_id.clone())
        .data(data.clone())
        .data_format("application/json".to_string())
        .schema(schema_url)
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    let read = actor
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));
}
