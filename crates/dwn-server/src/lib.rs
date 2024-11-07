//! DWN HTTP server, using [axum](https://github.com/tokio-rs/axum).
//!
//! The DWN spec does not define a standard HTTP API, so this server is simply built
//! to be compatible with the [dwn](https://github.com/unavi-xyz/dwn/tree/main/crates/dwn)
//! crate.
//!
//! ## Design
//!
//! The server provides a REST API, leaning into the strengths of HTTP.
//! For example, using HTTP-level status codes instead of the spec-defined
//! JSON reply objects.

use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::put,
    Json, Router,
};
use axum_macros::debug_handler;
use dwn::{core::message::Message, Dwn};
use tracing::debug;
use xdid::core::did::Did;

pub use dwn::core::reply::Reply;

pub fn create_router(dwn: Dwn) -> Router {
    Router::new()
        .route("/:target", put(handle_put))
        .with_state(dwn)
}

#[debug_handler]
async fn handle_put(
    Path(target): Path<String>,
    State(dwn): State<Dwn>,
    Json(msg): Json<Message>,
) -> Result<Json<Option<Reply>>, StatusCode> {
    let target = Did::from_str(&target).map_err(|e| {
        debug!("Failed to parse DID: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    let reply = dwn.process_message(&target, msg).await?;

    Ok(Json(reply))
}
