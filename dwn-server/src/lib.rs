use std::net::SocketAddr;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use dwn::data::{Message, RecordIdGenerator};
use tracing::{error, info, span, warn};

pub struct StartOptions {
    pub port: u16,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self { port: 3000 }
    }
}

/// Start the server.
pub async fn start(StartOptions { port }: StartOptions) {
    let app = Router::new().route("/", post(post_handler));

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

// https://identity.foundation/decentralized-web-node/spec/#request-level-status-coding
const DID_NOT_FOUND: (StatusCode, &str) = (StatusCode::NOT_FOUND, "DID not found");
const SERVER_ERROR: (StatusCode, &str) = (
    StatusCode::INTERNAL_SERVER_ERROR,
    "The request could not be processed correctly",
);

async fn post_handler(body: Json<dwn::data::RequestBody>) -> Response {
    for message in body.0.messages.iter() {
        if let Err(e) = process_message(message) {
            warn!("{}", e);
            return SERVER_ERROR.into_response();
        }
    }

    StatusCode::OK.into_response()
}

fn process_message(message: &Message) -> Result<(), Box<dyn std::error::Error>> {
    span!(tracing::Level::INFO, "message", ?message);

    {
        // Validate record_id
        let generator = RecordIdGenerator::try_from(&message.descriptor)?;
        let cid = generator.generate_cid()?;

        if cid != message.record_id {
            return Err("Record ID not valid".into());
        }
    }

    if message.data.is_some() {
        match &message.descriptor.data_cid {
            Some(_) => {
                // TODO: Validate data_cid
            }
            None => {
                return Err("Message has data but dataCid is None".into());
            }
        };

        match &message.descriptor.data_format {
            Some(_) => {
                // TODO: Validate data_format
            }
            None => {
                return Err("Message has data but dataFormat is None".into());
            }
        };
    }

    // Process message
    info!("Received message: {:?}", message);

    Ok(())
}
