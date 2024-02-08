use std::{net::SocketAddr, sync::Arc};

use dwn_server::{router, AppState};
use sqlx::mysql::MySqlPoolOptions;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create connection pool");

    let app = router(Arc::new(AppState { pool }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind port");

    info!("Listening on port {}", addr.port());

    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
    }
}
