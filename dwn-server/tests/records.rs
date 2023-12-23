use dwn::request::{descriptor::records::RecordsWrite, message::Message, RequestBody};
use dwn_test_utils::{expect_status, spawn_server};
use reqwest::StatusCode;

#[tokio::test]
async fn records_write() {
    let port = spawn_server().await;

    let body = RequestBody {
        messages: vec![Message::new(RecordsWrite::default())],
    };

    expect_status(body, port, StatusCode::OK).await;
}
