//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! The DWN spec is a work-in-progress and often out of date from other implementations,
//! so it is treated more as a loose guide rather than an absolute set of rules to follow.

use std::sync::Arc;

use dwn_core::{
    message::{Interface, Message, Method},
    reply::Reply,
    store::{DataStore, RecordStore},
};
use reqwest::StatusCode;
use tracing::debug;
use xdid::core::did::Did;

pub use dwn_core as core;

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
    pub async fn process_message(
        &self,
        target: &Did,
        msg: Message,
    ) -> Result<Option<Reply>, StatusCode> {
        debug!(
            "Processing {} {}",
            msg.descriptor.interface.to_string(),
            msg.descriptor.method.to_string()
        );

        if let Err(e) = handlers::validation::validate_message(target, &msg).await {
            debug!("Failed to validate message: {:?}", e);
            return Err(StatusCode::BAD_REQUEST);
        };

        let res = match &msg.descriptor.interface {
            Interface::Records => match &msg.descriptor.method {
                Method::Read => {
                    handlers::records::read::handle(self.record_store.as_ref(), target, msg)
                        .map(|v| Some(Reply::RecordsRead(Box::new(v))))?
                }
                Method::Query => {
                    handlers::records::query::handle(self.record_store.as_ref(), target, msg)
                        .await
                        .map(|v| Some(Reply::RecordsQuery(v)))?
                }
                Method::Write => {
                    handlers::records::write::handle(self.record_store.as_ref(), target, msg)
                        .await?;
                    None
                }
                Method::Subscribe => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                Method::Delete => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                _ => return Err(StatusCode::BAD_REQUEST),
            },
            Interface::Protocols => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            Interface::Permissions => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };

        Ok(res)
    }
}
