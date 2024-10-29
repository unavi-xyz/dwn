use dwn_core::{
    message::{DateSort, Filter, Message},
    store::{RecordStore, RecordStoreError},
};
use tracing::{debug, warn};
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

    fn query(
        &self,
        target: &Did,
        filter: &Filter,
        authorized: bool,
    ) -> Result<Vec<Message>, RecordStoreError> {
        debug!("querying {}", target);

        if filter.protocol.is_some() {
            debug_assert!(filter.protocol_version.is_some());
        }

        let tx = self
            .0
            .r_transaction()
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?;

        let mut found = tx
            .scan()
            .primary::<Record>()
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?
            .start_with((target.to_string(), "".to_string()))
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?
            .filter(|r| {
                let Ok(r) = r else {
                    warn!("Failed to read record during scan {}", target);
                    return false;
                };

                if !authorized && (r.message.descriptor.published != Some(true)) {
                    return false;
                }

                if let Some(attester) = &filter.attester {
                    match &r.message.attestation {
                        Some(jws) => {
                            if !jws.signatures.iter().any(|s| s.header.kid.did == *attester) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }

                if let Some(_recipient) = &filter.recipient {
                    // TODO
                }

                if let Some(schema) = filter.schema.as_deref() {
                    if r.message.descriptor.schema.as_deref() != Some(schema) {
                        return false;
                    }
                }

                if let Some(record_id) = filter.record_id.as_deref() {
                    if r.message.record_id != record_id {
                        return false;
                    }
                }

                if let Some(parent_id) = filter.parent_id.as_deref() {
                    if r.message.descriptor.parent_id.as_deref() != Some(parent_id) {
                        return false;
                    }
                }

                if let Some(context_id) = filter.context_id.as_deref() {
                    if r.message.context_id.as_deref() != Some(context_id) {
                        return false;
                    }
                }

                if let Some(protocol) = filter.protocol.as_deref() {
                    let version = filter.protocol_version.as_ref().unwrap();

                    if r.message.descriptor.protocol.as_deref() != Some(protocol) {
                        return false;
                    }

                    if r.message.descriptor.protocol_version.as_ref() == Some(version) {
                        return false;
                    }
                }

                if let Some(data_format) = &filter.data_format {
                    if r.message.descriptor.data_format.as_ref() != Some(data_format) {
                        return false;
                    }
                }

                if let Some(date_created) = &filter.date_created {
                    if r.message.descriptor.date_created < date_created.from {
                        return false;
                    }
                    if r.message.descriptor.date_created > date_created.to {
                        return false;
                    }
                }

                true
            })
            .map(|r| r.unwrap().message)
            .collect::<Vec<_>>();

        if let Some(date_sort) = &filter.date_sort {
            found.sort_by(|a, b| match date_sort {
                DateSort::Ascending => a.descriptor.date_created.cmp(&b.descriptor.date_created),
                DateSort::Descending => b.descriptor.date_created.cmp(&a.descriptor.date_created),
            });
        }

        Ok(found)
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
