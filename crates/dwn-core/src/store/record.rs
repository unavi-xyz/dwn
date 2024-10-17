use thiserror::Error;

use crate::message::Message;

pub trait RecordStore: Send + Sync {
    fn read(&self, target: &str, record_id: &str) -> Result<Option<Message>, RecordStoreError>;
    fn write(&self, target: &str, message: Message) -> Result<(), RecordStoreError>;
}

#[derive(Error, Debug)]
pub enum RecordStoreError {
    #[error("backend error")]
    BackendError,
}
