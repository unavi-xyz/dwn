use serde::{Deserialize, Serialize};

use crate::message::Message;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Reply {
    RecordsQuery(RecordsQueryReply),
    RecordsRead(Box<RecordsReadReply>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct RecordsQueryReply {
    pub entries: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct RecordsReadReply {
    pub entry: Option<Message>,
}
