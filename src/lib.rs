use axum::{routing::post, Router};
use tracing::{error, info};

pub async fn server() {
    let app = Router::new().route(
        "/",
        post(|| async {
            info!("Got a request!");
            "Hello, World!"
        }),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
    }
}
