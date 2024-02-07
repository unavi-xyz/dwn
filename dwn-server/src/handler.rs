use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use dwn::{
    request::{descriptor::Descriptor, message::Message, RequestBody},
    response::{MessageResult, ResponseBody, Status},
};
use sqlx::MySqlPool;
use tracing::{info, span, warn};

use crate::{model, AppState};

pub async fn post(State(state): State<Arc<AppState>>, Json(body): Json<RequestBody>) -> Response {
    let pool = &state.pool;

    let iter = body.messages.iter().map(|message| async move {
        match process_message(message, pool).await {
            Ok(result) => result,
            Err(e) => {
                warn!("Error processing message: {}", e);
                MessageResult::error(e.to_string())
            }
        }
    });

    let mut replies = Vec::new();

    for result in iter {
        replies.push(result.await);
    }

    Json(ResponseBody {
        status: Some(Status::new(StatusCode::OK.as_u16(), None)),
        replies: Some(replies),
    })
    .into_response()
}

pub async fn process_message(message: &Message, pool: &MySqlPool) -> Result<MessageResult> {
    span!(tracing::Level::INFO, "message", ?message);

    match message.descriptor {
        Descriptor::RecordsWrite(_) => {
            match &message.authorization {
                Some(auth) => {
                    let (_header, _payload) = auth.decode().await?;
                }
                None => return Ok(MessageResult::unauthorized()),
            };

            let entry_id = message.generate_record_id()?;

            if entry_id == message.record_id {
                match sqlx::query_as!(model::Record, "SELECT * FROM Record WHERE id = 'test'")
                    .fetch_one(pool)
                    .await
                {
                    Ok(record) => {
                        info!("Record: {:?}", record);
                    }
                    Err(_) => {
                        info!("No record found, creating new record");
                    }
                }
            }

            Ok(MessageResult::ok())
        }
        _ => Ok(MessageResult::error("Unsupported descriptor".to_string())),
    }
}
