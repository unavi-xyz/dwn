use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use dwn::{
    request::{descriptor::Descriptor, message::Message, RequestBody},
    response::{MessageResult, MessageStatus, ResponseBody},
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
                    Ok(MessageResult {
                        entries: None,
                        status: MessageStatus::ok(),
                    })
                }
                Err(_) => Ok(MessageResult {
                    entries: None,
                    status: MessageStatus::ok(),
                }),
            }
        }
        Descriptor::RecordsWrite(_) => {
            match &message.authorization {
                Some(auth) => {
                    let _ = auth.decode_verify().await?;
                }
                None => {
                    return Ok(MessageResult {
                        entries: None,
                        status: MessageStatus::unauthorized(),
                    })
                }
            };

            let entry_id = message.generate_record_id()?;

            if entry_id == message.record_id {
                match sqlx::query!("SELECT id FROM RecordsWrite WHERE id = ?", entry_id)
                    .fetch_one(pool)
                    .await
                {
                    Ok(_) => {
                        // Inital entry already exists, cease processing
                        return Ok(MessageResult {
                            entries: None,
                            status: MessageStatus::ok(),
                        });
                    }
                    Err(_) => {
                        // Store as initial entry
                        sqlx::query!(
                            "INSERT INTO RecordsWrite (id, data) VALUES (?, ?)",
                            entry_id,
                            message.data.as_ref().unwrap()
                        )

                        // TODO: Store RecordsWrite message in db
                        // TODO: Store data in S3
                    }
                }
            } else {
                // TODO: Process parent_id
            }

            Ok(MessageResult {
                entries: None,
                status: MessageStatus::ok(),
            })
        }
        _ => Ok(MessageResult {
            entries: None,
            status: MessageStatus::interface_not_implemented(),
        }),
    }
}
