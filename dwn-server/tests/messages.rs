use dwn::data::{DescriptorBuilder, Message, MessageBuilder, RequestBody};
use dwn_server::StartOptions;
use reqwest::StatusCode;

fn spawn_server() -> u16 {
    let port = port_check::free_local_port().expect("Failed to find free port");

    tokio::spawn(async move {
        dwn_server::start(StartOptions { port }).await;
    });

    // Wait for server to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    port
}

async fn send_post(data: dwn::data::RequestBody, port: u16) -> StatusCode {
    let client = reqwest::Client::new();

    let res = match client
        .post(format!("http://localhost:{}", port))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&data).expect("Failed to serialize data"))
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

    res.status()
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

    assert_eq!(send_post(body, port).await, StatusCode::OK);
}

#[tokio::test]
async fn requires_valid_record_id() {
    let port = spawn_server();

    let mut msg = empty_message();
    msg.record_id = "invalid record id".to_string();

    let body = RequestBody {
        messages: vec![msg],
    };

    assert_eq!(
        send_post(body, port).await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn requires_data_descriptors() {
    let port = spawn_server();

    let mut msg = empty_message();
    msg.data = Some("test data".to_string());
    msg.descriptor.data_cid = Some("test data cid".to_string());
    msg.descriptor.data_format = Some("test data format".to_string());

    let body = RequestBody {
        messages: vec![msg],
    };

    let mut without_cid = body.clone();
    without_cid.messages[0].descriptor.data_cid = None;

    let mut without_format = body.clone();
    without_format.messages[0].descriptor.data_format = None;

    let mut without_both = body.clone();
    without_both.messages[0].descriptor.data_cid = None;
    without_both.messages[0].descriptor.data_format = None;

    assert_eq!(
        send_post(without_cid, port).await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        send_post(without_format, port).await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        send_post(without_both, port).await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}
