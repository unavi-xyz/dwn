use dwn_core::{
    message::{
        descriptor::{DateSort, Descriptor, Filter},
        Message,
    },
    store::{Record, RecordStore, RecordStoreError},
};
use tracing::{debug, error, warn};
use xdid::core::did::Did;

use crate::{
    data::{InitialEntry, LatestEntry},
    NativeDbStore,
};

impl RecordStore for NativeDbStore<'_> {
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
            .primary::<LatestEntry>()
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?
            .start_with((target.to_string(), "".to_string()))
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?
            .filter(|res| {
                let Ok(entry) = res.as_ref().map(|r| &r.entry) else {
                    warn!("Failed to read record during scan {}", target);
                    return false;
                };

                let Descriptor::RecordsWrite(desc) = &entry.descriptor else {
                    panic!("invalid descriptor: {:?}", entry.descriptor);
                };

                if !authorized && (desc.published != Some(true)) {
                    return false;
                }

                if let Some(attester) = &filter.attester {
                    match &entry.attestation {
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
                    if desc.schema.as_deref() != Some(schema) {
                        return false;
                    }
                }

                if let Some(record_id) = filter.record_id.as_deref() {
                    if entry.record_id != record_id {
                        return false;
                    }
                }

                if let Some(protocol) = filter.protocol.as_deref() {
                    let version = filter.protocol_version.as_ref().unwrap();

                    if desc.protocol.as_deref() != Some(protocol) {
                        return false;
                    }

                    if desc.protocol_version.as_ref() == Some(version) {
                        return false;
                    }
                }

                if let Some(data_format) = &filter.data_format {
                    if desc.data_format.as_ref() != Some(data_format) {
                        return false;
                    }
                }

                if let Some(date_created) = &filter.date_created {
                    if desc.message_timestamp < date_created.from {
                        return false;
                    }
                    if desc.message_timestamp > date_created.to {
                        return false;
                    }
                }

                true
            })
            .map(|r| r.unwrap().entry)
            .collect::<Vec<_>>();

        found.sort_by(|a, b| match filter.date_sort.unwrap_or_default() {
            DateSort::Ascending => a
                .descriptor
                .message_timestamp()
                .cmp(b.descriptor.message_timestamp()),
            DateSort::Descending => b
                .descriptor
                .message_timestamp()
                .cmp(a.descriptor.message_timestamp()),
        });

        Ok(found)
    }

    fn read(&self, target: &Did, record_id: &str) -> Result<Option<Record>, RecordStoreError> {
        debug!("reading {} {}", target, record_id);

        let tx = self
            .0
            .r_transaction()
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?;

        let Some(initial_entry) = tx
            .get()
            .primary::<InitialEntry>((target.to_string(), record_id))
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?
            .map(|v| v.entry)
        else {
            return Ok(None);
        };

        let Some(latest_entry) = tx
            .get()
            .primary::<LatestEntry>((target.to_string(), record_id))
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?
            .map(|v| v.entry)
        else {
            error!("Found initial entry with no latest entry.");
            return Ok(None);
        };

        Ok(Some(Record {
            initial_entry,
            latest_entry,
        }))
    }

    fn write(&self, target: &Did, message: Message) -> Result<(), RecordStoreError> {
        debug!("writing {} {}", target, message.record_id);

        let tx = self
            .0
            .rw_transaction()
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?;

        let prev = tx
            .upsert(LatestEntry {
                key: (target.to_string(), message.record_id.clone()),
                entry: message.clone(),
            })
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?;

        if prev.is_none() {
            debug_assert_eq!(
                message.record_id,
                message.descriptor.compute_entry_id().unwrap()
            );

            tx.insert(InitialEntry {
                key: (target.to_string(), message.record_id.clone()),
                entry: message,
            })
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| RecordStoreError::BackendError(e.to_string()))?;

        Ok(())
    }
}
