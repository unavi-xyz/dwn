use std::future::Future;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

use crate::{
    message::Message,
    store::{DataStoreError, MessageStoreError},
    util::EncodeError,
};

pub mod records;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("Invalid descriptor")]
    InvalidDescriptor(String),
    #[error(transparent)]
    DataStoreError(#[from] DataStoreError),
    #[error(transparent)]
    MessageStoreError(#[from] MessageStoreError),
    #[error(transparent)]
    CborEncode(#[from] EncodeError),
}

pub trait MethodHandler {
    fn handle(
        &self,
        message: Message,
    ) -> impl Future<Output = Result<impl Into<Reply>, HandlerError>>;
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
    pub fn ok() -> Self {
        Status {
            code: 200,
            detail: Some(String::from("OK")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Reply {
    RecordsQuery(RecordsQueryReply),
    RecordsRead(Box<RecordsReadReply>),
    Status(StatusReply),
}

impl Reply {
    pub fn status(&self) -> &Status {
        match self {
            Reply::RecordsQuery(reply) => &reply.status,
            Reply::RecordsRead(reply) => &reply.status,
            Reply::Status(reply) => &reply.status,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordsQueryReply {
    pub entries: Vec<Message>,
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordsReadReply {
    pub data: Option<Vec<u8>>,
    pub record: Message,
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StatusReply {
    pub status: Status,
}

impl From<RecordsQueryReply> for Reply {
    fn from(val: RecordsQueryReply) -> Self {
        Reply::RecordsQuery(val)
    }
}

impl From<RecordsReadReply> for Reply {
    fn from(val: RecordsReadReply) -> Self {
        Reply::RecordsRead(Box::new(val))
    }
}

impl From<StatusReply> for Reply {
    fn from(val: StatusReply) -> Self {
        Reply::Status(val)
    }
}
