use std::str::FromStr;

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use dwn_core::{
    message::{data::Data, descriptor::Descriptor, mime::APPLICATION_JSON, Message},
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
    debug_assert!(matches!(msg.descriptor, Descriptor::RecordsWrite(_)));

    if msg.authorization.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let computed_entry_id = msg.descriptor.compute_entry_id().map_err(|e| {
        debug!("Failed to compute entry id: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    let Descriptor::RecordsWrite(desc) = &msg.descriptor else {
        panic!("invalid descriptor: {:?}", msg.descriptor);
    };

    let latest_entry = records.read(target, &msg.record_id).map_err(|e| {
        debug!("Failed to read record id {}: {:?}", msg.record_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if computed_entry_id == msg.record_id {
        if latest_entry.is_some() {
            // Entry already exists.
            return Ok(());
        }
    } else if let Some(prev) = &latest_entry {
        // Ensure immutable values remain unchanged.
        let Descriptor::RecordsWrite(initial_desc) = &prev.initial_entry.descriptor else {
            error!("Initial entry not RecordsWrite: {:?}", prev.initial_entry);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };

        if desc.schema != initial_desc.schema {
            debug!(
                "Schema does not match: {:?} != {:?}",
                desc.schema, initial_desc.schema
            );
            return Err(StatusCode::BAD_REQUEST);
        }

        // Ensure the message is newer than the stored entry.
        // If the dates match, compare the entry ids lexicographically.
        if desc.message_timestamp < *prev.latest_entry.descriptor.message_timestamp() {
            debug!(
                "Message created after stored entry: {} < {}",
                desc.message_timestamp,
                prev.latest_entry.descriptor.message_timestamp()
            );
            return Err(StatusCode::CONFLICT);
        }

        let prev_id = prev
            .latest_entry
            .descriptor
            .compute_entry_id()
            .map_err(|e| {
                error!(
                    "Failed to compute entry id for stored entry {}: {:?}",
                    prev.latest_entry.record_id, e
                );
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if (desc.message_timestamp == *prev.latest_entry.descriptor.message_timestamp())
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
    if let Some(schema_url) = &desc.schema {
        if desc.data_format != Some(APPLICATION_JSON) {
            debug!(
                "Message has schema, but data format is not application/json: {:?}",
                desc.data_format
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
