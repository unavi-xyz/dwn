use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use dwn::{
    request::{
        data::{Data, JsonData},
        descriptor::Descriptor,
        message::Message,
        RequestBody,
    },
    response::{MessageResult, ResponseBody},
};
use sqlx::MySqlPool;
use tracing::{span, warn};

use crate::AppState;

pub async fn post(State(state): State<Arc<AppState>>, Json(body): Json<RequestBody>) -> Response {
    let mut replies = Vec::new();

    for result in body.messages {
        match process_message(&result, &state.pool).await {
            Ok(result) => replies.push(result),
            Err(e) => {
                warn!("Error processing message: {}", e);
                return Json(ResponseBody::error()).into_response();
            }
        }
    }

    Json(ResponseBody {
        replies: Some(replies),
        status: None,
    })
    .into_response()
}

pub async fn process_message(message: &Message, pool: &MySqlPool) -> Result<MessageResult> {
    span!(tracing::Level::INFO, "message", ?message);

    match message.descriptor {
        Descriptor::RecordsRead(_) => {
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
        Descriptor::RecordsWrite(_) => {
            match &message.authorization {
                Some(auth) => {
                    let _ = auth.decode_verify().await?;
                }
                None => {
                    return Ok(MessageResult::unauthorized());
                }
            };

            let entry_id = message.generate_record_id()?;

            if entry_id == message.record_id {
                match sqlx::query!(
                    "SELECT entry_id FROM RecordsWrite WHERE entry_id = ?",
                    entry_id
                )
                .fetch_one(pool)
                .await
                {
                    Ok(_) => {
                        // Inital entry already exists, cease processing
                        return Ok(MessageResult::ok(None));
                    }
                    Err(_) => {
                        // Store message as initial entry
                        let data = match &message.data {
                            Some(data) => data.as_ref(),
                            None => {
                                warn!("Data not provided");
                                return Ok(MessageResult::bad_request());
                            }
                        };

                        let data = match JsonData::try_from_base64url(data) {
                            Ok(data) => data,
                            Err(_) => {
                                warn!("Data not valid");
                                return Ok(MessageResult::bad_request());
                            }
                        };

                        let data_cid = data.data_cid();

                        // TODO: Store data in S3

                        let descriptor_cid = message.descriptor.cid().to_bytes();

                        let descriptor = match &message.descriptor {
                            Descriptor::RecordsWrite(descriptor) => descriptor,
                            _ => {
                                warn!("Descriptor not provided");
                                return Ok(MessageResult::bad_request());
                            }
                        };

                        let data_format = data.data_format().to_string();
                        let published = descriptor.published.unwrap_or_default();
                        let record_id = message.record_id.to_string();

                        sqlx::query!(
                            "INSERT INTO RecordsWrite (entry_id, descriptor_cid, data_cid, data_format, published, record_id) VALUES (?, ?, ?, ?, ?, ?)",
                            entry_id,
                            descriptor_cid,
                            data_cid,
                            data_format,
                            published,
                            record_id,
                        )
                        .execute(pool)
                        .await?;
                    }
                }
            } else {
                // TODO: Process parent_id
            }

            Ok(MessageResult::ok(None))
        }
        _ => Ok(MessageResult::interface_not_implemented()),
    }
}
