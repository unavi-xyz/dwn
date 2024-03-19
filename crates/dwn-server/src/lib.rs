//! Decentralized Web Node HTTP server, using [axum](https://github.com/tokio-rs/axum).

use axum::Router;

pub fn router() -> Router {
    Router::new()
}
