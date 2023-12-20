//! Utility functions used in tests.

use dwn::{request::RequestBody, response::ResponseBody};
use reqwest::{Response, StatusCode};

/// Starts a DWN server on a random open port and returns the port.
pub fn spawn_server() -> u16 {
    let port = port_check::free_local_port().expect("Failed to find free port");

    tokio::spawn(async move {
        dwn_server::start(dwn_server::StartOptions { port }).await;
    });

    // Wait for server to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    port
}

/// Sends a JSON post request to the server.
pub async fn send_post(data: RequestBody, port: u16) -> Response {
    let client = reqwest::Client::new();

    client
        .post(format!("http://localhost:{}", port))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&data).expect("Failed to serialize data"))
        .send()
        .await
        .expect("Failed to send request")
}

/// Sends a request to the server and asserts that each reply has the expected status code.
pub async fn expect_status(body: RequestBody, port: u16, status: StatusCode) {
    let res = send_post(body, port)
        .await
        .json::<ResponseBody>()
        .await
        .expect("Failed to parse response body");

    for reply in res.replies.unwrap().iter() {
        assert_eq!(reply.status.code, status);
    }
}
