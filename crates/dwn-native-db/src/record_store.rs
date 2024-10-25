use dwn_core::{
    message::Message,
    store::{RecordStore, RecordStoreError},
};
use tracing::debug;
use xdid::core::did::Did;

use crate::{data::Record, NativeDbStore};

impl RecordStore for NativeDbStore<'_> {
    fn read(
        &self,
        target: &Did,
        record_id: &str,
        authorized: bool,
    ) -> Result<Option<Message>, RecordStoreError> {
        debug!("reading {} {}", target, record_id);

        let tx = self
            .0
            .r_transaction()
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?;

        let Some(value) = tx
            .get()
            .primary::<Record>((target.to_string(), record_id))
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?
        else {
            return Ok(None);
        };

        if !authorized && value.message.descriptor.published != Some(true) {
            return Ok(None);
        }

        Ok(Some(value.message))
    }

    fn write(&self, target: &Did, message: Message) -> Result<(), RecordStoreError> {
        debug!("writing {} {}", target, message.record_id);

        let tx = self
            .0
            .rw_transaction()
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?;

        tx.upsert(Record {
            key: (target.to_string(), message.record_id.clone()),
            message,
        })
        .map_err(|e| RecordStoreError::BackendError(e.to_string()))?;

        tx.commit()
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?;

        Ok(())
    }
}
