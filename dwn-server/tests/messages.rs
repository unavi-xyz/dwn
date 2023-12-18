use dwn::data::{DescriptorBuilder, Message, MessageBuilder, RequestBody};
use reqwest::StatusCode;

const SERVER_ADDR: &str = "http://localhost:3000";

fn spawn_server() {
    tokio::spawn(async move {
        dwn_server::start().await;
    });

    // Wait for server to start
    std::thread::sleep(std::time::Duration::from_secs(2));
}

async fn send_post(data: dwn::data::RequestBody) -> StatusCode {
    let client = reqwest::Client::new();

    let res = match client
        .post(SERVER_ADDR)
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
    spawn_server();

    let body = RequestBody {
        messages: vec![empty_message()],
    };

    assert_eq!(send_post(body).await, StatusCode::OK);
}

#[tokio::test]
async fn requires_valid_record_id() {
    spawn_server();

    let mut msg = empty_message();
    msg.record_id = "invalid record id".to_string();

    let body = RequestBody {
        messages: vec![msg],
    };

    assert_eq!(send_post(body).await, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn requires_data_descriptors() {
    spawn_server();

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
        send_post(without_cid).await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        send_post(without_format).await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        send_post(without_both).await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}
