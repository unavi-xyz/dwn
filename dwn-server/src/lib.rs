use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use dwn::{
    request::{message::Descriptor, Message, RequestBody},
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
    let replies = body
        .0
        .messages
        .iter()
        .map(process_message)
        .map(|result| match result {
            Ok(message_result) => message_result,
            Err(e) => {
                warn!("Failed to process message: {}", e);
                MessageResult {
                    entries: None,
                    status: Status::new(
                        StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        Some("The request could not be processed correctly"),
                    ),
                }
            }
        })
        .collect::<Vec<_>>();

    Json(ResponseBody {
        status: Some(Status::new(StatusCode::OK.as_u16(), None)),
        replies: Some(replies),
    })
    .into_response()
}

fn process_message(message: &Message) -> Result<MessageResult, Box<dyn std::error::Error>> {
    span!(tracing::Level::INFO, "message", ?message);

    match message {
        Message::RecordsWrite(message) => {
            let entry_id = message.descriptor.record_id().unwrap();

            // "IF Initial Entry exists for a record, store the entry as the Initial Entry for the record
            //  IF no Initial Entry exists and cease any further processing."
            // https://identity.foundation/decentralized-web-node/spec/#initial-record-entry
            //
            // THIS MAKES NO SENSE TO ME. HOW DO YOU CREATE THE INITIAL ENTRY???
            // I'm just going to assume the spec is wrong and got the logic backwards.
            if entry_id == message.record_id {}

            Ok(MessageResult {
                status: Status::new(StatusCode::OK.as_u16(), None),
                entries: None,
            })
        }
        _ => Err("Unsupported message type".into()),
    }
}
