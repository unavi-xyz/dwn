use dwn::request::RequestBody;
use dwn_test_utils::{send_post, spawn_server};
use reqwest::StatusCode;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn recieve_post() {
    let port = spawn_server().await;

    let body = RequestBody::new(Vec::new());
    let res = send_post(body, port).await;

    assert_eq!(res.status(), StatusCode::OK);
}
