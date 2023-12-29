use dwn::request::{descriptor::records::RecordsWrite, message::Message, RequestBody};
use dwn_test_utils::{expect_status, gen_auth, spawn_server};
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
    msg.authorization = Some(gen_auth(&msg).await);

    // Valid message
    {
        let body = RequestBody::new(vec![msg]);
        expect_status(body, port, StatusCode::OK).await;
    }
}
