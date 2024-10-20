use dwn::{builders::records::write::RecordsWriteBuilder, DWN};
use dwn_core::message::mime::APPLICATION_JSON;
use dwn_native_db::NativeDbStore;
use serde_json::json;
use tracing::info;
use tracing_test::traced_test;

use crate::utils::serve_string;

use super::expect_success;

#[tokio::test]
#[traced_test]
async fn test_write_schema() {
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = DWN::from(store);

    let target = "did:example:123";

    let schema = json!({ "maxLength": 5 });
    let data = json!("foo");
    assert!(jsonschema::is_valid(&schema, &data));

    let schema_url = serve_string(schema.to_string()).await;
    info!("Serving schema at {}", schema_url);

    let msg = RecordsWriteBuilder::default()
        .data(APPLICATION_JSON, data.to_string().as_bytes().to_owned())
        .schema(schema_url)
        .build()
        .unwrap();

    expect_success(&dwn, msg, target).await;
}
