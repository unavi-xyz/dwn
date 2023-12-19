use dwn::{
    data::JsonData,
    features::FeatureDetection,
    request::{DescriptorBuilder, Interface, Message, MessageBuilder, Method, RequestBody},
    response::ResponseBody,
};
use dwn_server::StartOptions;
use reqwest::{Response, StatusCode};
use serde_json::json;

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
    let builder = MessageBuilder::<JsonData> {
        data: None,
        descriptor: DescriptorBuilder {
            method: Method::FeatureDetectionRead,
            interface: Interface::FeatureDetection,
        },
    };

    builder.build().expect("Failed to build message")
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
async fn recieve_post() {
    let port = spawn_server();

    let body = RequestBody {
        messages: vec![empty_message()],
    };

    let res = send_post(body, port).await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn feature_detection() {
    let port = spawn_server();

    let body = RequestBody {
        messages: vec![empty_message()],
    };

    let res = send_post(body, port)
        .await
        .json::<ResponseBody>()
        .await
        .expect("Failed to parse response body");

    let replies = match res.replies {
        Some(replies) => replies,
        None => panic!("No replies in response body"),
    };

    assert_eq!(replies.len(), 1);

    let reply = &replies[0];

    assert_eq!(reply.status.code, StatusCode::OK);

    let entries = match reply.entries.as_ref() {
        Some(entries) => entries,
        None => panic!("No entries in reply"),
    };

    assert_eq!(entries.len(), 1);

    let entry = entries[0].clone();

    serde_json::from_value::<FeatureDetection>(entry).expect("Failed to parse feature detection");
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

        expect_status(body, port, StatusCode::INTERNAL_SERVER_ERROR).await;
    }
}

#[tokio::test]
async fn requires_data_descriptors() {
    let port = spawn_server();

    let msg = MessageBuilder::<JsonData>::new(
        Interface::FeatureDetection,
        Method::FeatureDetectionRead,
        JsonData(json!({
            "foo": "bar",
        })),
    )
    .build()
    .expect("Failed to build message");

    // Ensure base message is valid
    {
        let body = RequestBody {
            messages: vec![msg.clone()],
        };
        expect_status(body, port, StatusCode::OK).await;
    }

    let mut without_cid = msg.clone();
    without_cid.descriptor.data_cid = None;
    without_cid.record_id = without_cid.generate_record_id().unwrap();

    let mut without_format = msg.clone();
    without_format.descriptor.data_format = None;
    without_format.record_id = without_format.generate_record_id().unwrap();

    let mut without_both = msg.clone();
    without_both.descriptor.data_cid = None;
    without_both.descriptor.data_format = None;
    without_both.record_id = without_both.generate_record_id().unwrap();

    let body = RequestBody {
        messages: vec![without_cid, without_format, without_both],
    };

    expect_status(body, port, StatusCode::INTERNAL_SERVER_ERROR).await;
}
