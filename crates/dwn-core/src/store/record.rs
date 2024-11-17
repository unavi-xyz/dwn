use serde::{Deserialize, Serialize};
use thiserror::Error;
use xdid::core::did::Did;

use crate::message::{descriptor::Filter, Message};

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub initial_entry: Message,
    pub latest_entry: Message,
}

pub trait RecordStore: Send + Sync {
    fn query(
        &self,
        target: &Did,
        filter: &Filter,
        authorized: bool,
    ) -> Result<Vec<Message>, RecordStoreError>;

    fn read(&self, target: &Did, record_id: &str) -> Result<Option<Record>, RecordStoreError>;

    fn write(&self, target: &Did, message: Message) -> Result<(), RecordStoreError>;
}

#[derive(Error, Debug)]
pub enum RecordStoreError {
    #[error("backend error: {0}")]
    BackendError(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
