use dwn::{builders::records::write::RecordsWriteBuilder, Dwn};
use dwn_core::message::mime::{APPLICATION_JSON, TEXT_PLAIN};
use dwn_native_db::NativeDbStore;
use serde_json::json;
use tracing_test::traced_test;

use crate::utils::serve_string;

use super::{expect_fail, expect_success};

#[tokio::test]
#[traced_test]
async fn test_schema_success() {
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);

    let target = "did:example:123";

    let schema = json!({ "maxLength": 5 });
    let data = json!("foo");
    assert!(jsonschema::is_valid(&schema, &data));

    let schema_url = serve_string(schema.to_string()).await;

    let msg = RecordsWriteBuilder::default()
        .data(APPLICATION_JSON, data.to_string().as_bytes().to_owned())
        .schema(schema_url)
        .build()
        .unwrap();

    expect_success(&dwn, msg, target).await;
}

#[tokio::test]
#[traced_test]
async fn test_schema_fail() {
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);

    let target = "did:example:123";

    let schema = json!({ "maxLength": 2 });
    let data = json!("foo");
    assert!(!jsonschema::is_valid(&schema, &data));

    let schema_url = serve_string(schema.to_string()).await;

    let msg = RecordsWriteBuilder::default()
        .data(APPLICATION_JSON, data.to_string().as_bytes().to_owned())
        .schema(schema_url)
        .build()
        .unwrap();

    expect_fail(&dwn, msg, target).await;
}

#[tokio::test]
#[traced_test]
async fn test_invalid_schema() {
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);

    let target = "did:example:123";

    let schema = "not a valid schema";
    let data = json!("foo");

    let schema_url = serve_string(schema.to_string()).await;

    let msg = RecordsWriteBuilder::default()
        .data(APPLICATION_JSON, data.to_string().as_bytes().to_owned())
        .schema(schema_url)
        .build()
        .unwrap();

    expect_fail(&dwn, msg, target).await;
}

#[tokio::test]
#[traced_test]
async fn test_invalid_schema_url() {
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);

    let target = "did:example:123";

    let data = json!("foo");
    let schema_url = "not a url".to_string();

    let msg = RecordsWriteBuilder::default()
        .data(APPLICATION_JSON, data.to_string().as_bytes().to_owned())
        .schema(schema_url)
        .build()
        .unwrap();

    expect_fail(&dwn, msg, target).await;
}

#[tokio::test]
#[traced_test]
async fn test_schema_requires_data_format_json() {
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);

    let target = "did:example:123";

    let schema = json!({ "maxLength": 5 });
    let data = json!("foo");
    assert!(jsonschema::is_valid(&schema, &data));

    let schema_url = serve_string(schema.to_string()).await;

    let msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data.to_string().as_bytes().to_owned())
        .schema(schema_url)
        .build()
        .unwrap();

    expect_fail(&dwn, msg, target).await;
}
