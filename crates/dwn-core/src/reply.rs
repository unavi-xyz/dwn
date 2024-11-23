use serde::{Deserialize, Serialize};

use crate::{message::Message, store::Record};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Reply {
    RecordsSync(Box<RecordsSyncReply>),
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct RecordsSyncReply {
    /// Records that have conflicting latest entries.
    pub conflict: Vec<Message>,
    /// Records only the local has.
    pub local_only: Vec<String>,
    /// Records only the remote has.
    pub remote_only: Vec<Record>,
}
