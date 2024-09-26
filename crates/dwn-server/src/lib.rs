//! [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/) HTTP server.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use dwn::{message::DwnRequest, DWN};
use tracing::warn;

pub fn router(dwn: DWN) -> Router {
    Router::new().route("/", post(handle_post)).with_state(dwn)
}

async fn handle_post(State(dwn): State<DWN>, Json(request): Json<DwnRequest>) -> Response {
    match dwn.process_message(request).await {
        Ok(reply) => Json(reply).into_response(),
        Err(err) => {
            warn!("Error processing message: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
