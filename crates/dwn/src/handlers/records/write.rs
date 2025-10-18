use std::str::FromStr;

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use dwn_core::message::{
    data::Data,
    descriptor::{Can, Descriptor, ProtocolStructure, RecordFilter, Who},
    mime::APPLICATION_JSON,
};
use reqwest::StatusCode;
use serde_json::Value;
use tracing::{debug, error, warn};

use crate::ProcessContext;

pub async fn handle(
    ProcessContext {
        rs,
        ds,
        validation,
        target,
        msg,
    }: ProcessContext<'_>,
) -> Result<(), StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::RecordsWrite(_)));

    let mut authenticated = validation.authenticated.contains(target);

    let computed_entry_id = msg.descriptor.compute_entry_id().map_err(|e| {
        debug!("Failed to compute entry id: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    let Descriptor::RecordsWrite(desc) = &msg.descriptor else {
        panic!("invalid descriptor: {:?}", msg.descriptor);
    };

    let latest_entry = rs.read(ds, target, &msg.record_id).map_err(|e| {
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

        if desc.protocol != initial_desc.protocol {
            debug!(
                "Protocol does not match: {:?} != {:?}",
                desc.protocol, initial_desc.protocol
            );
            return Err(StatusCode::BAD_REQUEST);
        }

        if desc.protocol_path != initial_desc.protocol_path {
            debug!(
                "Protocol path does not match: {:?} != {:?}",
                desc.protocol_path, initial_desc.protocol_path
            );
            return Err(StatusCode::BAD_REQUEST);
        }

        if desc.protocol_version != initial_desc.protocol_version {
            debug!(
                "Protocol version does not match: {:?} != {:?}",
                desc.protocol_version, initial_desc.protocol_version
            );
            return Err(StatusCode::BAD_REQUEST);
        }

        // Ensure the message is newer than the stored entry.
        // If the dates match, compare the entry ids lexicographically.
        if desc.message_timestamp < *prev.latest_entry.descriptor.message_timestamp().unwrap() {
            debug!(
                "Message created after stored entry: {} < {}",
                desc.message_timestamp,
                prev.latest_entry.descriptor.message_timestamp().unwrap()
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

        if (desc.message_timestamp == *prev.latest_entry.descriptor.message_timestamp().unwrap())
            && (computed_entry_id < prev_id)
        {
            return Ok(());
        }
    } else {
        // Message is not the initial entry, and no initial entry was found.
        debug!("Initial entry not found for: {}", msg.record_id);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate protocol.
    if let Some(protocol) = &desc.protocol {
        let Some(version) = &desc.protocol_version else {
            debug!("Protocol version not supplied");
            return Err(StatusCode::BAD_REQUEST);
        };

        let Some(path) = &desc.protocol_path else {
            debug!("Protocol path not supplied");
            return Err(StatusCode::BAD_REQUEST);
        };

        let definition =
            match rs.query_protocol(target, protocol.clone(), vec![version.clone()], true) {
                Ok(found) => match found.into_iter().next().map(|x| x.1) {
                    Some(d) => d,
                    None => {
                        debug!("Protocol {protocol} not found");
                        return Err(StatusCode::NOT_FOUND);
                    }
                },
                Err(e) => {
                    debug!("Could not find protocol: {e}");
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

        let mut structure: Option<&ProtocolStructure> = None;
        let parts = path.split("/").collect::<Vec<_>>();

        for part in &parts {
            let structures = match structure {
                Some(s) => &s.children,
                None => &definition.structure,
            };

            let Some(s) = structures.get(*part) else {
                debug!("Invalid path: {path}");
                return Err(StatusCode::BAD_REQUEST);
            };

            structure = Some(s);
        }

        let Some(structure) = structure else {
            debug!("Invalid path: {path}");
            return Err(StatusCode::BAD_REQUEST);
        };

        let Some(actions) = &structure.actions else {
            debug!("No structure actions: {path}");
            return Err(StatusCode::BAD_REQUEST);
        };

        // TODO: Validate full context ID path
        // TODO: Enforce max context depth

        let can = if latest_entry.is_some() {
            Can::Update
        } else {
            Can::Create
        };

        let mut can_write = false;

        for action in actions {
            if action.can != can {
                continue;
            }

            match action.who {
                Who::Anyone => {
                    can_write = true;
                    break;
                }
                Who::Author => {
                    let of_sigs = if let Some(of) = &action.of {
                        let mut of_i = None;

                        for (i, prev) in parts.iter().enumerate().rev() {
                            if prev == of {
                                of_i = Some(i);
                                break;
                            }
                        }

                        let Some(of_i) = of_i else {
                            continue;
                        };

                        let Some(context_id) = &msg.context_id else {
                            continue;
                        };

                        let Some(of_id) = context_id.split("/").nth(of_i) else {
                            debug!("Invalid context id");
                            return Err(StatusCode::BAD_REQUEST);
                        };

                        let target = match rs.query(
                            target,
                            &RecordFilter {
                                record_id: Some(of_id.to_string()),
                                ..Default::default()
                            },
                            true,
                        ) {
                            Ok(res) => match res.into_iter().next() {
                                Some(m) => m,
                                None => {
                                    debug!("Target record {of_id} not found");
                                    return Err(StatusCode::NOT_FOUND);
                                }
                            },
                            Err(e) => {
                                debug!("Could not find target record: {e}");
                                return Err(StatusCode::BAD_REQUEST);
                            }
                        };

                        target
                            .authorization
                            .map(|a| a.signatures)
                            .unwrap_or_default()
                    } else if let Some(entry) = &latest_entry {
                        [&entry.initial_entry, &entry.latest_entry]
                            .into_iter()
                            .flat_map(|m| m.authorization.as_ref().map(|a| a.signatures.clone()))
                            .flatten()
                            .collect::<Vec<_>>()
                    } else {
                        continue;
                    };

                    for sig in of_sigs {
                        if validation.authenticated.contains(&sig.header.kid.did) {
                            can_write = true;
                            break;
                        }
                    }
                }
                Who::Recipient => {
                    if validation.authenticated.contains(target) {
                        can_write = true;
                        break;
                    }
                }
            }
        }

        if !can_write {
            debug!("Cannot write according to protocol rules");
            return Err(StatusCode::BAD_REQUEST);
        }

        authenticated = true;
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
            debug!("Schema is not an HTTP URL: {schema_url}");
            return Err(StatusCode::BAD_REQUEST);
        }

        let schema = reqwest::get(schema_url)
            .await
            .map_err(|e| {
                debug!("Failed to fetch schema {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .json::<Value>()
            .await
            .map_err(|e| {
                debug!("Failed to parse schema {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let validator = jsonschema::validator_for(&schema).map_err(|e| {
            debug!("Failed to create schema validator: {e:?}");
            StatusCode::BAD_REQUEST
        })?;

        let value = match &msg.data {
            Some(Data::Base64(d)) => {
                let decoded = BASE64_URL_SAFE_NO_PAD.decode(d).map_err(|e| {
                    debug!("Failed to base64 decode data: {e:?}");
                    StatusCode::BAD_REQUEST
                })?;
                let utf8 = String::from_utf8(decoded).map_err(|e| {
                    debug!("Failed to parse data as utf8: {e:?}");
                    StatusCode::BAD_REQUEST
                })?;
                Value::from_str(&utf8).map_err(|e| {
                    debug!("Failed to parse data as JSON: {e:?}");
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

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Err(e) = rs.write(ds, target, msg) {
        warn!("Error during write: {e:?}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(())
}
