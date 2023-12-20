use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use dwn::{
    request::{Message, RequestBody},
    response::{MessageResult, ResponseBody, Status},
};
use std::net::SocketAddr;
use tracing::{error, info, span, warn};

#[cfg(test)]
pub mod test_utils;

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
                    status: Status::new(
                        StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        Some("The request could not be processed correctly"),
                    ),
                    entries: None,
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

    // Validate record_id
    // {
    //     let generator = RecordIdGenerator::try_from(&message.descriptor)?;
    //     let cid = generator.generate_cid()?;
    //
    //     if cid != message.record_id {
    //         return Err("Record ID not valid".into());
    //     }
    // }

    match message {
        Message::RecordsWrite(message) => {
            info!("Processing RecordsWrite message {:?}", message);

            Ok(MessageResult {
                status: Status::new(StatusCode::OK.as_u16(), None),
                entries: None,
            })
        }
        _ => Err("Unsupported message type".into()),
    }
}
