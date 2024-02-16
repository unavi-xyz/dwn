use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use surrealdb::sql::Thing;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateEncodedMessage {
    pub(super) cid: String,
    pub(super) message: Vec<u8>,
    pub(super) tenant: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetEncodedMessage {
    pub(super) cid: String,
    pub(super) message: Vec<u8>,
    pub(super) id: Thing,
    pub(super) tenant: String,
}
