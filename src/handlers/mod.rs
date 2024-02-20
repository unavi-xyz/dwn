use std::future::Future;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

use crate::{
    message::{Message, VerifyAuthError},
    store::{DataStoreError, MessageStoreError},
    util::EncodeError,
};

pub mod records;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("Failed to verify message: {0}")]
    VerifyError(#[from] VerifyAuthError),
    #[error("Invalid descriptor")]
    InvalidDescriptor(String),
    #[error("Failed to interact with data store: {0}")]
    DataStoreError(#[from] DataStoreError),
    #[error("Failed to interact with message store: {0}")]
    MessageStoreError(#[from] MessageStoreError),
    #[error("CBOR encoding error: {0}")]
    CborEncode(#[from] EncodeError),
}

pub trait MethodHandler {
    fn handle(
        &self,
        tenant: &str,
        message: Message,
    ) -> impl Future<Output = Result<Reply, HandlerError>>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    pub status: Option<Status>,
    pub replies: Vec<Reply>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Status {
    pub code: u16,
    pub detail: Option<String>,
}

impl Status {
    fn ok() -> Self {
        Status {
            code: 200,
            detail: Some(String::from("OK")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Reply {
    RecordsQuery {
        entries: Vec<Message>,
        status: Status,
    },
    RecordsRead {
        data: Vec<u8>,
        record: Message,
        status: Status,
    },
    Status {
        status: Status,
    },
}

impl Reply {
    pub fn status(&self) -> &Status {
        match self {
            Reply::RecordsQuery { status, .. } => status,
            Reply::RecordsRead { status, .. } => status,
            Reply::Status { status } => status,
        }
    }
}
