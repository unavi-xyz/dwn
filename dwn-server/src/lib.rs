use anyhow::Result;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use dwn::{
    request::{descriptor::Descriptor, message::Message, RequestBody},
    response::{MessageResult, ResponseBody, Status},
};
use std::net::SocketAddr;
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

async fn post_handler(body: Json<RequestBody>) -> Response {
    let iter = body.0.messages.iter().map(|message| async move {
        let result = process_message(message).await;
        match result {
            Ok(result) => result,
            Err(e) => {
                warn!("Error processing message: {}", e);
                MessageResult::error(e.to_string())
            }
        }
    });

    let mut replies = Vec::new();

    for result in iter {
        replies.push(result.await);
    }

    Json(ResponseBody {
        status: Some(Status::new(StatusCode::OK.as_u16(), None)),
        replies: Some(replies),
    })
    .into_response()
}

async fn process_message(message: &Message) -> Result<MessageResult> {
    span!(tracing::Level::INFO, "message", ?message);

    match message.descriptor {
        Descriptor::RecordsWrite(_) => {
            match &message.authorization {
                Some(auth) => {
                    let (_header, _payload) = auth.decode().await?;
                }
                None => return Ok(MessageResult::unauthorized()),
            };

            let entry_id = message.generate_record_id()?;

            // "IF Initial Entry exists for a record, store the entry as the Initial Entry for the record
            //  IF no Initial Entry exists and cease any further processing."
            // https://identity.foundation/decentralized-web-node/spec/#initial-record-entry
            //
            // THIS MAKES NO SENSE TO ME. HOW DO YOU CREATE THE INITIAL ENTRY???
            // I'm just going to assume the spec is wrong and got the logic backwards.
            if entry_id == message.record_id {
                info!("Creating initial entry for record {}", message.record_id);
                // TODO: Create initial entry in database
            }

            Ok(MessageResult::ok())
        }
        _ => Ok(MessageResult::error("Unsupported descriptor".to_string())),
    }
}
