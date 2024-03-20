//! Decentralized Web Node HTTP server, using [axum](https://github.com/tokio-rs/axum).

use std::sync::Arc;

use axum::{routing::post, Json, Router};
use dwn::{
    message::Request,
    store::{DataStore, MessageStore},
    DWN,
};

pub fn router<D, M>(dwn: Arc<DWN<D, M>>) -> Router
where
    D: DataStore + Send + Sync + 'static,
    M: MessageStore + Send + Sync + 'static,
{
    Router::new().route(
        "/",
        post(
            |Json(request): Json<Request>| async move { Json(dwn.process_request(request).await) },
        ),
    )
}
