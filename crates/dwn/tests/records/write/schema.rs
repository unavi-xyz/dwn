use dwn_core::message::{
    descriptor::RecordsWriteBuilder,
    mime::{APPLICATION_JSON, TEXT_PLAIN},
};
use serde_json::json;
use tracing_test::traced_test;

use crate::utils::{init_dwn, serve_string};

use super::{expect_fail, expect_success};

#[tokio::test]
#[traced_test]
async fn test_schema_success() {
    let (actor, mut dwn) = init_dwn();

    let schema = json!({ "maxLength": 5 });
    let data = json!("foo");
    assert!(jsonschema::is_valid(&schema, &data));

    let schema_url = serve_string(schema.to_string()).await;

    let mut msg = RecordsWriteBuilder::default()
        .data(APPLICATION_JSON, data.to_string().into_bytes())
        .schema(schema_url)
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();

    expect_success(&actor.did, &mut dwn, msg).await;
}

#[tokio::test]
#[traced_test]
async fn test_schema_fail() {
    let (actor, mut dwn) = init_dwn();

    let schema = json!({ "maxLength": 2 });
    let data = json!("foo");
    assert!(!jsonschema::is_valid(&schema, &data));

    let schema_url = serve_string(schema.to_string()).await;

    let mut msg = RecordsWriteBuilder::default()
        .data(APPLICATION_JSON, data.to_string().into_bytes())
        .schema(schema_url)
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();

    expect_fail(&actor.did, &mut dwn, msg).await;
}

#[tokio::test]
#[traced_test]
async fn test_invalid_schema() {
    let (actor, mut dwn) = init_dwn();

    let schema = "not a valid schema";
    let data = json!("foo");

    let schema_url = serve_string(schema.to_string()).await;

    let mut msg = RecordsWriteBuilder::default()
        .data(APPLICATION_JSON, data.to_string().into_bytes())
        .schema(schema_url)
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();

    expect_fail(&actor.did, &mut dwn, msg).await;
}

#[tokio::test]
#[traced_test]
async fn test_invalid_schema_url() {
    let (actor, mut dwn) = init_dwn();

    let data = json!("foo");
    let schema_url = "not a url".to_string();

    let mut msg = RecordsWriteBuilder::default()
        .data(APPLICATION_JSON, data.to_string().into_bytes())
        .schema(schema_url)
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();

    expect_fail(&actor.did, &mut dwn, msg).await;
}

#[tokio::test]
#[traced_test]
async fn test_schema_requires_data_format_json() {
    let (actor, mut dwn) = init_dwn();

    let schema = json!({ "maxLength": 5 });
    let data = json!("foo");
    assert!(jsonschema::is_valid(&schema, &data));

    let schema_url = serve_string(schema.to_string()).await;

    let mut msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data.to_string().into_bytes())
        .schema(schema_url)
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();

    expect_fail(&actor.did, &mut dwn, msg).await;
}
