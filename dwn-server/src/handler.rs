use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use dwn::{
    request::{
        data::{Data, JsonData},
        descriptor::Descriptor,
        message::Message,
        RequestBody,
    },
    response::{MessageResult, ResponseBody},
};
use s3::{creds::Credentials, Bucket, Region};
use sqlx::{Executor, MySqlPool};
use tracing::{info_span, warn};

use crate::AppState;

pub async fn post(State(state): State<Arc<AppState>>, Json(body): Json<RequestBody>) -> Response {
    let mut replies = Vec::new();

    for result in body.messages {
        match process_message(&result, &state.pool).await {
            Ok(result) => replies.push(result),
            Err(e) => {
                warn!("Error processing message: {}", e);
                return Json(ResponseBody::error()).into_response();
            }
        }
    }

    Json(ResponseBody {
        replies: Some(replies),
        status: None,
    })
    .into_response()
}

pub async fn process_message(message: &Message, pool: &MySqlPool) -> Result<MessageResult> {
    info_span!("message", ?message);

    match message.descriptor {
        Descriptor::RecordsRead(_) => {
            match sqlx::query!(
                "SELECT data_cid FROM Record WHERE id = ?",
                &message.record_id
            )
            .fetch_one(pool)
            .await
            {
                Ok(_record) => {
                    // TODO: Fetch data_cid from S3

                    Ok(MessageResult::ok(None))
                }
                Err(_) => Ok(MessageResult::ok(None)),
            }
        }
        Descriptor::RecordsWrite(_) => {
            match &message.authorization {
                Some(auth) => {
                    let _ = auth.decode_verify().await?;
                }
                None => {
                    return Ok(MessageResult::unauthorized());
                }
            };

            let entry_id = message.generate_record_id()?;

            if entry_id == message.record_id {
                match sqlx::query!(
                    "SELECT entry_id FROM RecordsWrite WHERE entry_id = ?",
                    entry_id
                )
                .fetch_one(pool)
                .await
                {
                    Ok(_) => {
                        // Inital entry already exists, cease processing
                        return Ok(MessageResult::ok(None));
                    }
                    Err(_) => {
                        // Store message as initial entry
                        let data = match &message.data {
                            Some(data) => data.as_ref(),
                            None => {
                                warn!("Data not provided");
                                return Ok(MessageResult::bad_request());
                            }
                        };

                        let data = match JsonData::try_from_base64url(data) {
                            Ok(data) => data,
                            Err(_) => {
                                warn!("Data not valid");
                                return Ok(MessageResult::bad_request());
                            }
                        };

                        let data_cid = data.data_cid();

                        const S3_ACCESS_KEY_ID: &str = env!("S3_ACCESS_KEY_ID");
                        const S3_BUCKET_NAME: &str = env!("S3_BUCKET_NAME");
                        const S3_ENDPOINT: &str = env!("S3_ENDPOINT");
                        const S3_REGION: &str = env!("S3_REGION");
                        const S3_SECRET_ACCESS_KEY: &str = env!("S3_SECRET_ACCESS_KEY");

                        let region = Region::Custom {
                            region: S3_REGION.to_owned(),
                            endpoint: S3_ENDPOINT.to_owned(),
                        };
                        let credentials = Credentials {
                            access_key: Some(S3_ACCESS_KEY_ID.to_owned()),
                            expiration: None,
                            secret_key: Some(S3_SECRET_ACCESS_KEY.to_owned()),
                            security_token: None,
                            session_token: None,
                        };

                        let bucket = Bucket::new(S3_BUCKET_NAME, region.clone(), credentials)?
                            .with_path_style();

                        let s3_path = format!("records/{}.bin", data_cid);
                        let bytes = data.to_bytes();

                        let response_data = bucket.put_object(&s3_path, &bytes).await?;
                        assert_eq!(response_data.status_code(), 200);

                        let response_data = bucket.get_object(&s3_path).await?;
                        assert_eq!(response_data.status_code(), 200);
                        assert_eq!(bytes, response_data.as_slice());

                        let descriptor_cid = message.descriptor.cid().to_string();

                        let descriptor = match &message.descriptor {
                            Descriptor::RecordsWrite(descriptor) => descriptor,
                            _ => {
                                warn!("Descriptor not provided");
                                return Ok(MessageResult::bad_request());
                            }
                        };

                        let data_format = data.data_format().to_string();
                        let published = descriptor.published.unwrap_or_default();
                        let record_id = message.record_id.to_string();

                        let mut tx = pool.begin().await?;

                        tx.execute(sqlx::query!(
                            "INSERT INTO CidData (cid, path) VALUES (?, ?)",
                            data_cid,
                            s3_path,
                        ))
                        .await?;

                        tx.execute(
                            sqlx::query!(
                                "INSERT INTO RecordsWrite (entry_id, descriptor_cid, data_cid, data_format, published, record_id) VALUES (?, ?, ?, ?, ?, ?)",
                                entry_id,
                                descriptor_cid,
                                data_cid,
                                data_format,
                                published,
                                record_id,
                        ))
                        .await?;

                        tx.commit().await?;
                    }
                }
            } else {
                // TODO: Process parent_id
            }

            Ok(MessageResult::ok(None))
        }
        _ => Ok(MessageResult::interface_not_implemented()),
    }
}
