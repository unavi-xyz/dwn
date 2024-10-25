use thiserror::Error;
use xdid::core::did::Did;

use crate::message::Message;

pub trait RecordStore: Send + Sync {
    fn read(
        &self,
        target: &Did,
        record_id: &str,
        authorized: bool,
    ) -> Result<Option<Message>, RecordStoreError>;
    fn write(&self, target: &Did, message: Message) -> Result<(), RecordStoreError>;
}

#[derive(Error, Debug)]
pub enum RecordStoreError {
    #[error("backend error: {0}")]
    BackendError(String),
}
