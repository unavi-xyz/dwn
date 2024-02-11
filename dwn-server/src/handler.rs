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
use serde_json::Value;
use sqlx::MySqlPool;
use tracing::{info, span, warn};

use crate::{model, AppState};

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
        Descriptor::RecordsQuery(_) => {
            match sqlx::query_as!(
                model::Record,
                "SELECT * FROM Record WHERE id = ?",
                &message.record_id
            )
            .fetch_one(pool)
            .await
            {
                Ok(record) => {
                    info!("Found: {:?}", record);
                    Ok(MessageResult {
                        entries: Some(vec![Value::String(record.data)]),
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
                    let (_header, _payload) = auth.decode().await?;
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
                match sqlx::query_as!(model::Record, "SELECT * FROM Record WHERE id = ?", entry_id)
                    .fetch_one(pool)
                    .await
                {
                    Ok(record) => {
                        info!("Found existing record: {:?}", record);
                        return Ok(MessageResult {
                            entries: None,
                            status: MessageStatus::ok(),
                        });
                    }
                    Err(_) => {
                        info!("No record found, creating new record");

                        sqlx::query!(
                            "INSERT INTO Record (id, data) VALUES (?, ?)",
                            entry_id,
                            message.data.as_ref()
                        )
                        .execute(pool)
                        .await?;
                    }
                }
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
