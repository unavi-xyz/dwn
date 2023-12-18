use dwn::{
    request::{DescriptorBuilder, Message, MessageBuilder, RequestBody},
    response::ResponseBody,
};
use dwn_server::StartOptions;
use reqwest::{Response, StatusCode};

fn spawn_server() -> u16 {
    let port = port_check::free_local_port().expect("Failed to find free port");

    tokio::spawn(async move {
        dwn_server::start(StartOptions { port }).await;
    });

    // Wait for server to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    port
}

async fn send_post(data: RequestBody, port: u16) -> Response {
    let client = reqwest::Client::new();

    client
        .post(format!("http://localhost:{}", port))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&data).expect("Failed to serialize data"))
        .send()
        .await
        .expect("Failed to send request")
}

fn empty_message() -> Message {
    let builder = MessageBuilder {
        data: None,
        descriptor: DescriptorBuilder {
            method: "test method".to_string(),
            interface: "test interface".to_string(),
            data_format: None,
        },
    };

    builder.build().expect("Failed to build message")
}

#[tokio::test]
async fn recieve_post() {
    let port = spawn_server();

    let body = RequestBody {
        messages: vec![empty_message()],
    };

    let res = send_post(body, port).await;

    assert_eq!(res.status(), StatusCode::OK);
}

async fn expect_status(body: RequestBody, port: u16, status: StatusCode) {
    let res = send_post(body, port)
        .await
        .json::<ResponseBody>()
        .await
        .expect("Failed to parse response body");

    for reply in res.replies.unwrap().iter() {
        assert_eq!(reply.status.code, status);
    }
}

#[tokio::test]
async fn requires_valid_record_id() {
    let port = spawn_server();

    // Valid record ID
    {
        let body = RequestBody {
            messages: vec![empty_message()],
        };

        expect_status(body, port, StatusCode::OK).await;
    }

    // Invalid record ID
    {
        let mut msg = empty_message();
        msg.record_id = "invalid record id".to_string();

        let body = RequestBody {
            messages: vec![msg],
        };

        expect_status(body, port, StatusCode::BAD_REQUEST).await;
    }
}

#[tokio::test]
async fn requires_data_descriptors() {
    let port = spawn_server();

    let mut msg = empty_message();
    msg.data = Some("test data".to_string());
    msg.descriptor.data_cid = Some("test data cid".to_string());
    msg.descriptor.data_format = Some("test data format".to_string());

    let mut without_cid = msg.clone();
    without_cid.descriptor.data_cid = None;

    let mut without_format = msg.clone();
    without_format.descriptor.data_format = None;

    let mut without_both = msg.clone();
    without_both.descriptor.data_cid = None;

    let body = RequestBody {
        messages: vec![without_cid, without_format, without_both],
    };

    expect_status(body, port, StatusCode::BAD_REQUEST).await;
}
