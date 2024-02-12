use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use dwn::{
    request::{descriptor::Descriptor, message::Message, RequestBody},
    response::{MessageResult, ResponseBody},
};
use sqlx::MySqlPool;
use tracing::{info_span, warn};

use crate::records::{read::process_records_read, write::process_records_write};

mod records;

pub struct AppState {
    pub pool: MySqlPool,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", routing::post(post))
        .with_state(state)
}

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
    info_span!("message", ?message);

    match &message.descriptor {
        Descriptor::RecordsRead(_) => process_records_read(message, pool).await,
        Descriptor::RecordsWrite(_) => process_records_write(message, pool).await,
        _ => Ok(MessageResult::interface_not_implemented()),
    }
}
