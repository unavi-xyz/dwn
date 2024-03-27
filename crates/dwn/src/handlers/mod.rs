use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::message::Message;

pub mod records;

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
    RecordsRead(RecordsReadReply),
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
    pub record: Box<Message>,
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StatusReply {
    pub status: Status,
}

impl From<RecordsQueryReply> for Reply {
    fn from(reply: RecordsQueryReply) -> Self {
        Reply::RecordsQuery(reply)
    }
}

impl From<RecordsReadReply> for Reply {
    fn from(reply: RecordsReadReply) -> Self {
        Reply::RecordsRead(reply)
    }
}

impl From<StatusReply> for Reply {
    fn from(reply: StatusReply) -> Self {
        Reply::Status(reply)
    }
}
