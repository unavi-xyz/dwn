use dwn::{
    request::{
        data::{Data, JsonData},
        descriptor::records::{RecordsRead, RecordsWrite},
        message::Message,
        RequestBody,
    },
    response::ResponseBody,
};
use dwn_test_utils::{authorize, expect_status, gen_did, send_post, spawn_server};
use reqwest::StatusCode;
use sqlx::mysql::MySqlPoolOptions;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create connection pool");

    let port = spawn_server(pool).await;
    let (did, key) = gen_did();

    let record_id;

    // RecordsWrite
    {
        info!("Sending RecordsWrite message");

        let mut msg = Message::new(RecordsWrite::default());
        msg.authorization = Some(authorize(did, &key, &msg).await);

        let value = serde_json::json!({
            "foo": "bar",
            "baz": 42,
        });
        msg.data = Some(JsonData(value).to_base64url());

        record_id = msg.record_id.clone();

        let body = RequestBody::new(vec![msg.clone()]);
        expect_status(body, port, StatusCode::OK).await;
    }

    // RecordsRead
    {
        info!("Sending RecordsRead message");

        let mut msg = Message::new(RecordsRead::default());
        msg.record_id = record_id;

        let body = RequestBody::new(vec![msg]);
        let res = send_post(body, port)
            .await
            .json::<ResponseBody>()
            .await
            .expect("Failed to parse response body");

        for reply in res.replies.unwrap().iter() {
            assert_eq!(reply.status.code, StatusCode::OK);

            for entry in reply.entries.as_ref().unwrap().iter() {
                let json = serde_json::from_str::<serde_json::Value>(entry.as_str().unwrap())
                    .expect("Failed to parse entry as JSON");

                info!("Entry: {:?}", json);
            }
        }
    }
}
