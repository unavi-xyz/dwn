use std::sync::Arc;

use axum::{routing::post, Router};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};

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

pub async fn create_pool(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}
