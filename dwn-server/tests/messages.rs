use dwn::request::{records::RecordsWrite, RequestBody};
use dwn_test_utils::{expect_status, send_post, spawn_server};
use reqwest::StatusCode;

#[tokio::test]
async fn recieve_post() {
    let port = spawn_server();

    let body = RequestBody {
        messages: Vec::new(),
    };

    let res = send_post(body, port).await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn requires_valid_record_id() {
    let port = spawn_server();

    // Valid record ID
    {
        let body = RequestBody {
            messages: vec![RecordsWrite::default().into()],
        };

        expect_status(body, port, StatusCode::OK).await;
    }

    // Invalid record ID
    {
        let msg = RecordsWrite {
            record_id: "invalid".into(),
            ..RecordsWrite::default()
        };

        let body = RequestBody {
            messages: vec![msg.into()],
        };

        expect_status(body, port, StatusCode::INTERNAL_SERVER_ERROR).await;
    }
}
