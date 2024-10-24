use std::str::FromStr;

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use dwn_core::{
    message::{data::Data, mime::APPLICATION_JSON, Interface, Message, Method},
    store::RecordStore,
};
use serde_json::Value;
use tracing::debug;
use xdid::core::did::Did;

use crate::Status;

pub async fn handle(records: &dyn RecordStore, target: &Did, msg: Message) -> Result<(), Status> {
    debug_assert_eq!(msg.descriptor.interface, Interface::Records);
    debug_assert_eq!(msg.descriptor.method, Method::Write);

    let Ok(prev) = records.read(target, &msg.record_id) else {
        return Err(Status {
            code: 500,
            detail: "Internal error.",
        });
    };

    if let Some(prev) = &prev {
        if msg.descriptor.schema != prev.descriptor.schema {
            return Err(Status {
                code: 400,
                detail: "Invalid schema",
            });
        }
    }

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
            Some(Data::Encrypted(_)) => todo!("cannot validate encrypted data against schema"),
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

    match prev {
        Some(_) => {
            todo!("overwrite record");
        }
        None => {
            // For new records, the record ID must match the descriptor.
            match msg.descriptor.compute_record_id() {
                Ok(id) => {
                    if id != msg.record_id {
                        return Err(Status {
                            code: 400,
                            detail: "Invalid record ID",
                        });
                    }
                }
                Err(_) => {
                    return Err(Status {
                        code: 400,
                        detail: "Failed to compute record ID for message",
                    })
                }
            };

            if let Err(e) = records.write(target, msg) {
                debug!("Error during wring: {:?}", e);
                return Err(Status {
                    code: 500,
                    detail: "Internal error",
                });
            };

            Ok(())
        }
    }
}
