use dwn::request::RequestBody;
use dwn_test_utils::{send_post, spawn_server};
use reqwest::StatusCode;

#[tokio::test]
async fn recieve_post() {
    let port = spawn_server().await;

    let body = RequestBody {
        messages: Vec::new(),
    };

    let res = send_post(body, port).await;

    assert_eq!(res.status(), StatusCode::OK);
}
