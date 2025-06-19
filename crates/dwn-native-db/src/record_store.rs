use dwn_core::{
    message::{
        Message,
        descriptor::{DateSort, Descriptor, RecordFilter, RecordId, RecordsSync},
    },
    store::{DataStore, Record, RecordStore, StoreError},
};
use tracing::{debug, error, warn};
use xdid::core::did::Did;

use crate::{
    NativeDbStore,
    data::{InitialEntry, LatestEntry},
};

impl RecordStore for NativeDbStore<'_> {
    fn prepare_sync(&self, target: &Did, authorized: bool) -> Result<RecordsSync, StoreError> {
        debug!("syncing {}", target);

        let tx = self
            .0
            .r_transaction()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        let initial_entries = tx
            .scan()
            .primary::<InitialEntry>()
            .map_err(|e| StoreError::BackendError(e.to_string()))?
            .start_with((target.to_string(), "".to_string()))
            .map_err(|e| StoreError::BackendError(e.to_string()))?
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

                true
            })
            .map(|r| r.unwrap().entry)
            .collect::<Vec<_>>();

        let records = initial_entries
            .into_iter()
            .flat_map(|initial_entry| {
                tx.get()
                    .primary::<LatestEntry>((target.to_string(), initial_entry.record_id.clone()))
                    .map(|res| {
                        let Some(latest_entry) = res.map(|r| r.entry) else {
                            warn!(
                                "Latest entry not found for initial entry: {}",
                                initial_entry.record_id
                            );
                            return Err(StoreError::BackendError(
                                "Missing latest entry".to_string(),
                            ));
                        };

                        Ok(RecordId {
                            record_id: initial_entry.record_id,
                            latest_entry_id: latest_entry
                                .descriptor
                                .compute_entry_id()
                                .map_err(|e| StoreError::BackendError(e.to_string()))?,
                        })
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RecordsSync::new(records))
    }

    fn query(
        &self,
        target: &Did,
        filter: &RecordFilter,
        authorized: bool,
    ) -> Result<Vec<Message>, StoreError> {
        debug!("querying {}", target);

        if filter.protocol.is_some() {
            debug_assert!(filter.protocol_version.is_some());
        }

        let tx = self
            .0
            .r_transaction()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        let mut found = tx
            .scan()
            .primary::<LatestEntry>()
            .map_err(|e| StoreError::BackendError(e.to_string()))?
            .start_with((target.to_string(), "".to_string()))
            .map_err(|e| StoreError::BackendError(e.to_string()))?
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
                .cmp(&b.descriptor.message_timestamp()),
            DateSort::Descending => b
                .descriptor
                .message_timestamp()
                .cmp(&a.descriptor.message_timestamp()),
        });

        Ok(found)
    }

    fn read(
        &self,
        ds: &dyn DataStore,
        target: &Did,
        record_id: &str,
    ) -> Result<Option<Record>, StoreError> {
        debug!("reading {} {}", target, record_id);

        let tx = self
            .0
            .r_transaction()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        let Some(initial_entry) = tx
            .get()
            .primary::<InitialEntry>((target.to_string(), record_id))
            .map_err(|e| StoreError::BackendError(e.to_string()))?
            .map(|v| v.entry)
        else {
            return Ok(None);
        };

        let Some(mut latest_entry) = tx
            .get()
            .primary::<LatestEntry>((target.to_string(), record_id))
            .map_err(|e| StoreError::BackendError(e.to_string()))?
            .map(|v| v.entry)
        else {
            error!("Found initial entry with no latest entry.");
            return Ok(None);
        };

        if let Descriptor::RecordsWrite(desc) = &latest_entry.descriptor {
            if let Some(cid) = &desc.data_cid {
                latest_entry.data = ds.read(target, cid)?;
            }
        }

        Ok(Some(Record {
            initial_entry,
            latest_entry,
        }))
    }

    fn write(
        &self,
        ds: &dyn DataStore,
        target: &Did,
        mut message: Message,
    ) -> Result<(), StoreError> {
        debug!("writing {} {}", target, message.record_id);

        let tx = self
            .0
            .rw_transaction()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        let cid = if let Descriptor::RecordsWrite(desc) = &message.descriptor {
            desc.data_cid.clone()
        } else {
            None
        };

        let data = message.data.take();

        let prev = tx
            .upsert(LatestEntry {
                key: (target.to_string(), message.record_id.clone()),
                entry: message.clone(),
            })
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        if prev.is_none() {
            debug_assert_eq!(
                message.record_id,
                message.descriptor.compute_entry_id().unwrap()
            );

            tx.insert(InitialEntry {
                key: (target.to_string(), message.record_id.clone()),
                entry: message,
            })
            .map_err(|e| StoreError::BackendError(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        if let Some(cid) = cid {
            // TODO: We should use the same tx commit for more reliability

            // Add a reference for LatestEntry.
            ds.add_ref(target, &cid, data)?;

            // Remove previous reference.
            if let Some(prev) = prev {
                if let Descriptor::RecordsWrite(desc) = prev.entry.descriptor {
                    if let Some(prev_cid) = &desc.data_cid {
                        ds.remove_ref(target, prev_cid)?;
                    }
                }
            }
        }

        Ok(())
    }
}
