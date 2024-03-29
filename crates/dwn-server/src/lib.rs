//! Decentralized Web Node HTTP server, using [axum](https://github.com/tokio-rs/axum).

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use dwn::{
    message::Request,
    store::{DataStore, MessageStore},
    DWN,
};
use tracing::debug;

pub fn router(
    dwn: Arc<
        DWN<impl DataStore + Send + Sync + 'static, impl MessageStore + Send + Sync + 'static>,
    >,
) -> Router {
    Router::new().route("/", post(handle_post)).with_state(dwn)
}

async fn handle_post(
    State(dwn): State<
        Arc<DWN<impl DataStore + Send + Sync + 'static, impl MessageStore + Send + Sync + 'static>>,
    >,
    Json(request): Json<Request>,
) -> Response {
    match dwn.process_message(request).await {
        Ok(reply) => Json(reply).into_response(),
        Err(err) => {
            debug!("Error processing message: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
