use std::{collections::BTreeMap, net::SocketAddr};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use dwn::{
    features::FeatureDetection,
    request::{Message, Method, RecordIdGenerator, RequestBody},
    response::{MessageResult, ResponseBody, Status},
};
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

    match message.descriptor.method {
        Method::FeatureDetectionRead => {
            let mut features = FeatureDetection::default();

            features.interfaces.records = Some(BTreeMap::from_iter(vec![
                (Method::RecordsRead.to_string(), true),
                (Method::RecordsQuery.to_string(), true),
            ]));

            let value = serde_json::to_value(features)?;

            Ok(MessageResult::new(vec![value]))
        }
        _ => Err("Method not supported".into()),
    }
}
