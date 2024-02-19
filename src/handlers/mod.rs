use std::future::Future;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

use crate::{
    message::{Message, VerifyAuthError},
    store::{DataStoreError, MessageStoreError},
};

pub mod records;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("Failed to verify message: {0}")]
    VerifyError(#[from] VerifyAuthError),
    #[error("Invalid descriptor")]
    InvalidDescriptor,
    #[error("Failed to interact with data store: {0}")]
    DataStoreError(#[from] DataStoreError),
    #[error("Failed to interact with message store: {0}")]
    MessageStoreError(#[from] MessageStoreError),
}

pub trait MethodHandler {
    fn handle(
        &self,
        tenant: &str,
        message: Message,
    ) -> impl Future<Output = Result<MessageReply, HandlerError>>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageReply {
    pub status: Status,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Status {
    pub code: u16,
    pub detail: String,
}

impl Status {
    fn ok() -> Self {
        Status {
            code: 200,
            detail: "OK".to_string(),
        }
    }
}
