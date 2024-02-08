use std::sync::Arc;

use axum::{routing::post, Router};
use sqlx::MySqlPool;

mod handler;
mod model;

pub struct AppState {
    pub pool: MySqlPool,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(handler::post))
        .with_state(state)
}
