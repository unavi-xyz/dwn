use dwn::request::{
    data::{Data, JsonData},
    descriptor::records::RecordsWrite,
    message::Message,
    RequestBody,
};
use dwn_test_utils::{authorize, expect_status, gen_did, spawn_server};
use reqwest::StatusCode;
use sqlx::MySqlPool;
use tracing_test::traced_test;

#[sqlx::test]
#[traced_test]
fn records_write(pool: MySqlPool) {
    let port = spawn_server(pool).await;
    let (did, key) = gen_did();

    let mut msg_write = Message::new(RecordsWrite::default());

    let value = serde_json::json!({
        "foo": "bar",
        "baz": 42,
    });
    msg_write.data = Some(JsonData(value).to_base64url());

    // Require authorization
    {
        let body = RequestBody::new(vec![msg_write.clone()]);
        expect_status(body, port, StatusCode::UNAUTHORIZED).await;
    }

    msg_write.authorization = Some(authorize(did, &key, &msg_write).await);

    // Valid write
    {
        let body = RequestBody::new(vec![msg_write.clone()]);
        expect_status(body, port, StatusCode::OK).await;
    }
}
