use anyhow::Result;
use dwn::{request::message::Message, response::MessageResult};
use sqlx::MySqlPool;

pub async fn process_records_read(message: &Message, pool: &MySqlPool) -> Result<MessageResult> {
    match sqlx::query!(
        "SELECT data_cid FROM Record WHERE id = ?",
        &message.record_id
    )
    .fetch_one(pool)
    .await
    {
        Ok(_record) => {
            // TODO: Fetch data_cid from S3

            Ok(MessageResult::ok(None))
        }
        Err(_) => Ok(MessageResult::ok(None)),
    }
}
