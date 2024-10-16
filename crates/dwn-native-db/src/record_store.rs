use dwn_core::{
    message::Message,
    store::{RecordStore, RecordStoreError},
};

use crate::{data::Record, NativeDbStore};

impl RecordStore for NativeDbStore<'_> {
    fn read(&self, did: &str, record_id: &str) -> Result<Option<Message>, RecordStoreError> {
        let Ok(tx) = self.db.r_transaction() else {
            return Err(RecordStoreError::BackendError);
        };

        let Ok(value) = tx.get().primary::<Record>((did, record_id)) else {
            return Err(RecordStoreError::BackendError);
        };

        Ok(value.map(|v| v.message))
    }

    fn write(&self, did: &str, message: Message) -> Result<(), RecordStoreError> {
        let Ok(tx) = self.db.rw_transaction() else {
            return Err(RecordStoreError::BackendError);
        };

        tx.insert(Record {
            key: (did.to_owned(), message.record_id.clone()),
            message,
        })
        .map_err(|_| RecordStoreError::BackendError)?;

        Ok(())
    }
}
