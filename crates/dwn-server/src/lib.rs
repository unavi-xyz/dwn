//! DWN HTTP server, using [axum](https://github.com/tokio-rs/axum).
//!
//! The DWN spec does not define a standard HTTP API, so this server is simply built
//! to be compatible with the [dwn](https://github.com/unavi-xyz/dwn/tree/main/crates/dwn)
//! crate.

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
    // TODO: Anyone can write messages to the DWN, even if they are not `target`.
    // Is this a problem? We would need some authentication solution, verifying
    // the requester has permission to write messages.
    Router::new()
        .route("/:target", put(handle_put))
        .with_state(dwn)
}

#[debug_handler]
async fn handle_put(
    Path(target): Path<String>,
    State(mut dwn): State<Dwn>,
    Json(msg): Json<Message>,
) -> Result<Json<Option<Reply>>, StatusCode> {
    let target = Did::from_str(&target).map_err(|e| {
        debug!("Failed to parse DID: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    let reply = dwn.process_message(&target, msg).await?;

    Ok(Json(reply))
}
