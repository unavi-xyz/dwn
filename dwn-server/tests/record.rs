use dwn::request::{descriptor::records::RecordsWrite, message::Message, RequestBody};
use dwn_test_utils::{authorize, expect_status, gen_did, spawn_server};
use reqwest::StatusCode;
use sqlx::MySqlPool;
use tracing_test::traced_test;

#[sqlx::test]
#[traced_test]
fn require_auth(pool: MySqlPool) {
    let port = spawn_server(pool).await;
    let (did, key) = gen_did();

    let mut msg = Message::new(RecordsWrite::default());

    // Without authorization
    {
        let body = RequestBody::new(vec![msg.clone()]);
        expect_status(body, port, StatusCode::UNAUTHORIZED).await;
    }

    msg.authorization = Some(authorize(did, &key, &msg).await);

    // With authorization
    {
        let body = RequestBody::new(vec![msg]);
        expect_status(body, port, StatusCode::OK).await;
    }
}
