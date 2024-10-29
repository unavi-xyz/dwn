use std::str::FromStr;

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use dwn_core::{
    message::{data::Data, mime::APPLICATION_JSON, Interface, Message, Method},
    store::RecordStore,
};
use serde_json::Value;
use tracing::{debug, error};
use xdid::core::did::Did;

use crate::Status;

pub async fn handle(records: &dyn RecordStore, target: &Did, msg: Message) -> Result<(), Status> {
    debug_assert_eq!(msg.descriptor.interface, Interface::Records);
    debug_assert_eq!(msg.descriptor.method, Method::Write);

    if msg.authorization.is_none() {
        return Err(Status {
            code: 401,
            detail: "Unauthorized",
        });
    }

    let computed_record_id = msg.descriptor.compute_entry_id().map_err(|_| Status {
        code: 400,
        detail: "Failed to compute record id",
    })?;

    let Ok(prev) = records.read(target, &msg.record_id, true) else {
        return Err(Status {
            code: 500,
            detail: "Internal error",
        });
    };

    // 1. If the generated entry id matches the record id, and the initial entry
    //    already exists, cease processing (the spec says to do the opposite
    //    of this, but I think that is wrong).
    if computed_record_id == msg.record_id {
        if prev.is_some() {
            return Ok(());
        }
    } else {
        // 2. If mesesage is not the initial entry, parent id must be present.
        let Some(parent_id) = &msg.descriptor.parent_id else {
            return Err(Status {
                code: 400,
                detail: "Missing parent id",
            });
        };

        if let Some(prev) = &prev {
            // 3. Ensure all immutable values remain unchanged.
            if msg.descriptor.schema != prev.descriptor.schema {
                return Err(Status {
                    code: 400,
                    detail: "Invalid schema",
                });
            }

            // 4. Compare the parent id to the latest stored entry.
            //    If they do not match, cease processing.
            let prev_id = prev.descriptor.compute_entry_id().map_err(|_| {
                error!("Failed to compute record id for: {}", prev.record_id);
                Status {
                    code: 400,
                    detail: "Failed to compute record id for stored entry",
                }
            })?;

            if *parent_id != prev_id {
                return Ok(());
            }

            // 6. Ensure date created is greater than the stored entry.
            //    If the dates match, compare the entry ids lexicographically.
            if msg.descriptor.date_created < prev.descriptor.date_created {
                debug!("Message created after currently stored entry.");
                return Ok(());
            }

            if (msg.descriptor.date_created == prev.descriptor.date_created)
                && (computed_record_id < prev_id)
            {
                return Ok(());
            }
        }
    }

    // Validate data conforms to schema.
    if let Some(schema_url) = &msg.descriptor.schema {
        if msg.descriptor.data_format != Some(APPLICATION_JSON) {
            return Err(Status {
                code: 400,
                detail: "Data format must be application/json when using schemas",
            });
        }

        if !schema_url.starts_with("http") {
            return Err(Status {
                code: 400,
                detail: "Schema must be an HTTP URL",
            });
        }

        let schema = reqwest::get(schema_url)
            .await
            .map_err(|e| {
                debug!("Failed to fetch schema {:?}", e);
                Status {
                    code: 500,
                    detail: "Failed to fetch schema",
                }
            })?
            .json::<Value>()
            .await
            .map_err(|e| {
                debug!("Failed to parse schema {:?}", e);
                Status {
                    code: 500,
                    detail: "Failed to parse schema",
                }
            })?;

        let validator = jsonschema::validator_for(&schema).map_err(|e| {
            debug!("Failed to create schema validator: {:?}", e);
            Status {
                code: 400,
                detail: "Invalid schema",
            }
        })?;

        let value = match &msg.data {
            Some(Data::Base64(d)) => {
                let decoded = BASE64_URL_SAFE_NO_PAD.decode(d).map_err(|e| {
                    debug!("Failed to base64 decode data: {:?}", e);
                    Status {
                        code: 400,
                        detail: "Failed to base64 decode data",
                    }
                })?;
                let utf8 = String::from_utf8(decoded).map_err(|e| {
                    debug!("Failed to parse data as utf8: {:?}", e);
                    Status {
                        code: 400,
                        detail: "Failed to parse data as utf8",
                    }
                })?;
                Value::from_str(&utf8).map_err(|e| {
                    debug!("Failed to parse data as JSON: {:?}", e);
                    Status {
                        code: 400,
                        detail: "Data is not JSON",
                    }
                })?
            }
            Some(Data::Encrypted(_)) => {
                // TODO: Store the message without validation?
                return Err(Status {
                    code: 500,
                    detail: "Cannot validate schema for encrypted data.",
                });
            }
            None => {
                return Err(Status {
                    code: 400,
                    detail: "No data provided",
                })
            }
        };

        if !validator.is_valid(&value) {
            return Err(Status {
                code: 400,
                detail: "Data does not fulfill schema",
            });
        };
    }

    // 7. Store the inbound message as the latest entry.
    if let Err(e) = records.write(target, msg) {
        debug!("Error during write: {:?}", e);
        return Err(Status {
            code: 500,
            detail: "Internal error",
        });
    };

    Ok(())
}
