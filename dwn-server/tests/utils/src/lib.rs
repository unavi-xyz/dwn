//! Utility functions used in tests.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use dwn::{request::RequestBody, response::ResponseBody};
use reqwest::{Response, StatusCode};
use tokio::time::sleep;

/// Starts a DWN server on a random open port and returns the port.
pub async fn spawn_server() -> u16 {
    let port = port_check::free_local_port().expect("Failed to find free port");
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);

    tokio::spawn(async move {
        dwn_server::start(dwn_server::StartOptions { port }).await;
    });

    // Poll the port until it's open.
    while !port_check::is_port_reachable(addr) {
        sleep(std::time::Duration::from_millis(100)).await;
    }

    port
}

/// Sends a JSON post request to the server.
pub async fn send_post(data: RequestBody, port: u16) -> Response {
    let client = reqwest::Client::new();

    let url = format!("http://localhost:{}", port);

    client
        .post(&url)
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
