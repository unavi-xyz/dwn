use dwn_core::{
    message::Message,
    store::{RecordStore, RecordStoreError},
};
use tracing::debug;

use crate::{data::Record, NativeDbStore};

impl RecordStore for NativeDbStore<'_> {
    fn read(&self, target: &str, record_id: &str) -> Result<Option<Message>, RecordStoreError> {
        debug!("reading {}/{}", target, record_id);

        let Ok(tx) = self.0.r_transaction() else {
            return Err(RecordStoreError::BackendError);
        };

        let Ok(value) = tx.get().primary::<Record>((target, record_id)) else {
            return Err(RecordStoreError::BackendError);
        };

        Ok(value.map(|v| v.message))
    }

    fn write(&self, target: &str, message: Message) -> Result<(), RecordStoreError> {
        debug!("writing {}/{}", target, message.record_id);

        let Ok(tx) = self.0.rw_transaction() else {
            return Err(RecordStoreError::BackendError);
        };

        tx.insert(Record {
            key: (target.to_owned(), message.record_id.clone()),
            message,
        })
        .map_err(|_| RecordStoreError::BackendError)?;

        tx.commit().map_err(|_| RecordStoreError::BackendError)?;

        Ok(())
    }
}
