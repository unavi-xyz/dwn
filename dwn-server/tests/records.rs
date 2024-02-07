use dwn::request::{descriptor::records::RecordsWrite, message::Message, RequestBody};
use dwn_test_utils::{authorize, expect_status, gen_did, spawn_server};
use reqwest::StatusCode;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn records_write() {
    let port = spawn_server().await;

    let mut msg = Message::new(RecordsWrite::default());

    // Require authorization
    {
        let body = RequestBody::new(vec![msg.clone()]);
        expect_status(body, port, StatusCode::UNAUTHORIZED).await;
    }

    // Add authorization
    let (did, key) = gen_did();
    msg.authorization = Some(authorize(did, &key, &msg).await);

    // Valid message
    {
        let body = RequestBody::new(vec![msg]);
        expect_status(body, port, StatusCode::OK).await;
    }
}
