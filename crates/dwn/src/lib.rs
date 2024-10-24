//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! The DWN spec is a work-in-progress and often out of date from other implementations,
//! so it is treated more as a loose guide rather than an absolute set of rules to follow.

use std::sync::Arc;

use dwn_core::{
    message::{Interface, Message, Method},
    store::{DataStore, RecordStore},
};
use tracing::{debug, info_span};
use xdid::core::did::Did;

pub mod stores {
    #[cfg(feature = "native_db")]
    pub use dwn_native_db::*;
}

pub mod actor;
pub mod builders;
mod handlers;

#[derive(Clone)]
pub struct Dwn {
    pub data_store: Arc<dyn DataStore>,
    pub record_store: Arc<dyn RecordStore>,
}

impl<T: DataStore + RecordStore + Clone + 'static> From<T> for Dwn {
    fn from(value: T) -> Self {
        Self {
            data_store: Arc::new(value.clone()),
            record_store: Arc::new(value),
        }
    }
}

impl Dwn {
    pub async fn process_message(&self, target: &Did, msg: Message) -> Result<Reply, Status> {
        let span = {
            let interface = msg.descriptor.interface.to_string();
            let method = msg.descriptor.method.to_string();
            info_span!("process_message", interface, method).entered()
        };

        handlers::validation::validate_message(target, &msg)
            .await
            .map_err(|e| {
                debug!("Failed to validate message: {:?}", e);
                Status {
                    code: 400,
                    detail: "Failed validation.",
                }
            })?;

        let res = match &msg.descriptor.interface {
            Interface::Records => match &msg.descriptor.method {
                Method::Read => {
                    match handlers::records::read::handle(self.record_store.as_ref(), target, msg)?
                    {
                        Some(found) => Ok(Reply::RecordsRead(Box::new(found))),
                        None => Err(Status {
                            code: 404,
                            detail: "Not Found",
                        }),
                    }
                }
                Method::Query => Err(Status {
                    code: 500,
                    detail: "todo",
                }),
                Method::Write => {
                    handlers::records::write::handle(self.record_store.as_ref(), target, msg)
                        .await?;
                    Ok(Reply::Status(Status {
                        code: 200,
                        detail: "OK",
                    }))
                }
                Method::Subscribe => Err(Status {
                    code: 500,
                    detail: "todo",
                }),
                Method::Delete => Err(Status {
                    code: 500,
                    detail: "todo",
                }),
                _ => Err(Status {
                    code: 400,
                    detail: "Invalid descriptor method",
                }),
            },
            Interface::Protocols => Err(Status {
                code: 500,
                detail: "todo",
            }),
            Interface::Permissions => Err(Status {
                code: 500,
                detail: "todo",
            }),
        };

        drop(span);

        res
    }
}

pub enum Reply {
    RecordsQuery(Vec<Message>),
    RecordsRead(Box<Message>),
    Status(Status),
}

#[derive(Debug)]
pub struct Status {
    pub code: u64,
    pub detail: &'static str,
}
