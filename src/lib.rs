use std::net::SocketAddr;

use axum::{http::StatusCode, routing::post, Json, Router};
use tracing::{error, info, span, warn};

pub mod data;

pub async fn server() {
    let app = Router::new().route("/", post(post_handler));

    let port = 3000;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = match tokio::net::TcpListener::bind(addr).await {
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

async fn post_handler(body: Json<data::Body>) -> StatusCode {
    for message in body.0.messages.iter() {
        span!(tracing::Level::INFO, "message", ?message);

        // Validate message
        if message.data.is_some() {
            match &message.descriptor.data_cid {
                Some(_) => {
                    // TODO: Validate data_cid
                }
                None => {
                    warn!("Message has data but dataCid is None");
                    return StatusCode::BAD_REQUEST;
                }
            };

            match &message.descriptor.data_format {
                Some(_) => {
                    // TODO: Validate data_format
                }
                None => {
                    warn!("Message has data but dataFormat is None");
                    return StatusCode::BAD_REQUEST;
                }
            };
        }

        // Process message
        info!("Received message: {:?}", message);
    }

    StatusCode::OK
}
