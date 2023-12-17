use dwn::data::{Descriptor, Message};
use reqwest::StatusCode;

const SERVER_ADDR: &str = "http://localhost:3000";

fn spawn_server() {
    tokio::spawn(async move {
        dwn::server().await;
    });

    // Wait for server to start
    std::thread::sleep(std::time::Duration::from_millis(100));
}

async fn send_post(data: dwn::data::Body) -> StatusCode {
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

#[tokio::test]
async fn recieve_post() {
    spawn_server();

    let data = dwn::data::Body {
        messages: vec![Message {
            record_id: "test_record_id".to_string(),
            data: None,
            descriptor: Descriptor {
                method: "application/json".to_string(),
                interface: "test_interface".to_string(),
                data_cid: None,
                data_format: None,
            },
        }],
    };

    assert_eq!(send_post(data).await, StatusCode::OK);
}

#[tokio::test]
async fn requires_data_descriptors() {
    spawn_server();

    let data = dwn::data::Body {
        messages: vec![Message {
            record_id: "test_record_id".to_string(),
            data: Some("test_data".to_string()),
            descriptor: Descriptor {
                method: "application/json".to_string(),
                interface: "test_interface".to_string(),
                data_cid: Some("test_data_cid".to_string()),
                data_format: Some("test_data_format".to_string()),
            },
        }],
    };

    let mut without_cid = data.clone();
    without_cid.messages[0].descriptor.data_cid = None;

    let mut without_format = data.clone();
    without_format.messages[0].descriptor.data_format = None;

    let mut without_both = data.clone();
    without_both.messages[0].descriptor.data_cid = None;
    without_both.messages[0].descriptor.data_format = None;

    assert_eq!(send_post(without_cid).await, StatusCode::BAD_REQUEST);
    assert_eq!(send_post(without_format).await, StatusCode::BAD_REQUEST);
    assert_eq!(send_post(without_both).await, StatusCode::BAD_REQUEST);
}
