use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateData {
    pub(super) cid: String,
    pub(super) data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetData {
    pub(super) cid: String,
    pub(super) data: Vec<u8>,
    pub(super) id: Thing,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateMessage {
    pub(super) cid: String,
    pub(super) message: Vec<u8>,
    pub(super) tenant: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetMessage {
    pub(super) cid: String,
    pub(super) message: Vec<u8>,
    pub(super) id: Thing,
    pub(super) tenant: String,
}
