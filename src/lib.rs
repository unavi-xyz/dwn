use std::net::SocketAddr;

use axum::{routing::post, Json, Router};
use tracing::{error, info};

pub mod data;

pub async fn server() {
    let app = Router::new().route("/", post(post_handler));

    let port = 3000;

    let listener =
        match tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind to port {}: {}", port, e);
                return;
            }
        };

    info!("Listening on port {}", port);

    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
    }
}

async fn post_handler(body: Json<data::Body>) -> Result<String, ()> {
    info!("Received body: {:?}", body);
    Ok("OK".to_string())
}
