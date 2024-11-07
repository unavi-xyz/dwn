use std::str::FromStr;

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use dwn_core::{
    message::{data::Data, mime::APPLICATION_JSON, Interface, Message, Method},
    store::RecordStore,
};
use reqwest::StatusCode;
use serde_json::Value;
use tracing::{debug, error};
use xdid::core::did::Did;

pub async fn handle(
    records: &dyn RecordStore,
    target: &Did,
    msg: Message,
) -> Result<(), StatusCode> {
    debug_assert_eq!(msg.descriptor.interface, Interface::Records);
    debug_assert_eq!(msg.descriptor.method, Method::Write);

    if msg.authorization.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let computed_entry_id = msg.descriptor.compute_entry_id().map_err(|e| {
        debug!("Failed to compute entry id: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    let prev = records.read(target, &msg.record_id, true).map_err(|e| {
        debug!("Failed to read record id {}: {:?}", msg.record_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if computed_entry_id == msg.record_id {
        if prev.is_some() {
            // Entry already exists.
            return Ok(());
        }
    } else if let Some(prev) = &prev {
        // Ensure immutable values remain unchanged.
        if msg.descriptor.schema != prev.descriptor.schema {
            debug!(
                "Schema does not match: {:?} != {:?}",
                msg.descriptor.schema, prev.descriptor.schema
            );
            return Err(StatusCode::BAD_REQUEST);
        }

        // Ensure the message is newer than the stored entry.
        // If the dates match, compare the entry ids lexicographically.
        if msg.descriptor.message_timestamp < prev.descriptor.message_timestamp {
            debug!(
                "Message created after stored entry: {} < {}",
                msg.descriptor.message_timestamp, prev.descriptor.message_timestamp
            );
            return Err(StatusCode::CONFLICT);
        }

        let prev_id = prev.descriptor.compute_entry_id().map_err(|e| {
            error!(
                "Failed to compute entry id for stored entry {}: {:?}",
                prev.record_id, e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        if (msg.descriptor.message_timestamp == prev.descriptor.message_timestamp)
            && (computed_entry_id < prev_id)
        {
            return Ok(());
        }
    } else {
        // Message is not the initial entry, and no initial entry was found.
        debug!("Initial entry not found for: {}", msg.record_id);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate data conforms to schema.
    if let Some(schema_url) = &msg.descriptor.schema {
        if msg.descriptor.data_format != Some(APPLICATION_JSON) {
            debug!(
                "Message has schema, but data format is not application/json: {:?}",
                msg.descriptor.data_format
            );
            return Err(StatusCode::BAD_REQUEST);
        }

        if !schema_url.starts_with("http") {
            debug!("Schema is not an HTTP URL: {}", schema_url);
            return Err(StatusCode::BAD_REQUEST);
        }

        let schema = reqwest::get(schema_url)
            .await
            .map_err(|e| {
                debug!("Failed to fetch schema {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .json::<Value>()
            .await
            .map_err(|e| {
                debug!("Failed to parse schema {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let validator = jsonschema::validator_for(&schema).map_err(|e| {
            debug!("Failed to create schema validator: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;

        let value = match &msg.data {
            Some(Data::Base64(d)) => {
                let decoded = BASE64_URL_SAFE_NO_PAD.decode(d).map_err(|e| {
                    debug!("Failed to base64 decode data: {:?}", e);
                    StatusCode::BAD_REQUEST
                })?;
                let utf8 = String::from_utf8(decoded).map_err(|e| {
                    debug!("Failed to parse data as utf8: {:?}", e);
                    StatusCode::BAD_REQUEST
                })?;
                Value::from_str(&utf8).map_err(|e| {
                    debug!("Failed to parse data as JSON: {:?}", e);
                    StatusCode::BAD_REQUEST
                })?
            }
            Some(Data::Encrypted(_)) => {
                // TODO: Store the message without validation?
                return Err(StatusCode::BAD_REQUEST);
            }
            None => {
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        if !validator.is_valid(&value) {
            debug!("Data does not fulfill schema.");
            return Err(StatusCode::BAD_REQUEST);
        };
    }

    if let Err(e) = records.write(target, msg) {
        debug!("Error during write: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(())
}
