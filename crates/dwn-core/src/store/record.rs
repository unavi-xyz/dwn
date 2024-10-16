use crate::message::Message;

pub trait RecordStore: Send + Sync {
    fn read(&self, did: &str, record_id: &str) -> Result<Option<Message>, RecordStoreError>;
    fn write(&self, did: &str, message: Message) -> Result<(), RecordStoreError>;
}

pub enum RecordStoreError {
    BackendError,
}
