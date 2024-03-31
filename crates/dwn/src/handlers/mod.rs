use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::message::Message;

pub mod protocols;
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
#[serde(untagged)]
pub enum MessageReply {
    Query(QueryReply),
    RecordsRead(RecordsReadReply),
    Status(StatusReply),
}

impl MessageReply {
    pub fn status(&self) -> &Status {
        match self {
            MessageReply::Query(reply) => &reply.status,
            MessageReply::RecordsRead(reply) => &reply.status,
            MessageReply::Status(reply) => &reply.status,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryReply {
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

impl From<QueryReply> for MessageReply {
    fn from(reply: QueryReply) -> Self {
        MessageReply::Query(reply)
    }
}

impl From<RecordsReadReply> for MessageReply {
    fn from(reply: RecordsReadReply) -> Self {
        MessageReply::RecordsRead(reply)
    }
}

impl From<StatusReply> for MessageReply {
    fn from(reply: StatusReply) -> Self {
        MessageReply::Status(reply)
    }
}
