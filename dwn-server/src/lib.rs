use std::sync::Arc;

use axum::Router;
use sqlx::MySqlPool;

pub struct AppState {
    pub pool: MySqlPool,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new().with_state(state)
}
