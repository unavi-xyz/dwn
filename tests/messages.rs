use dwn::data::{Descriptor, Message};

const SERVER_ADDR: &str = "http://localhost:3000";

fn spawn_server() {
    tokio::spawn(async move {
        dwn::server().await;
    });

    // Wait for server to start
    std::thread::sleep(std::time::Duration::from_millis(100));
}

#[tokio::test]
async fn recieve_post() {
    spawn_server();

    let client = reqwest::Client::new();

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

    assert_eq!(res.status(), 200);
}
