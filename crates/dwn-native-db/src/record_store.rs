use dwn_core::{
    message::{
        Message,
        descriptor::{
            DateSort, Descriptor, ProtocolDefinition, RecordFilter, RecordId, RecordsSync,
        },
    },
    store::{DataStore, Record, RecordStore, StoreError},
};
use semver::VersionReq;
use tracing::{debug, error, warn};
use xdid::core::did::Did;

use crate::{
    NativeDbStore,
    data::{InitialEntry, LatestEntry, Protocol},
};

impl RecordStore for NativeDbStore<'_> {
    fn configure_protocol(&self, target: &Did, message: Message) -> Result<(), StoreError> {
        let Descriptor::ProtocolsConfigure(desc) = message.descriptor else {
            panic!("invalid message descriptor: {:?}", message.descriptor)
        };

        debug!("configuring protocol {}", desc.definition.protocol);

        let tx = self
            .0
            .rw_transaction()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        tx.upsert(Protocol {
            key: (target.to_string(), desc.definition.protocol.clone()),
            version: desc.protocol_version,
            definition: serde_json::to_vec(&desc.definition)
                .map_err(|e| StoreError::BackendError(e.to_string()))?,
        })
        .map_err(|e| StoreError::BackendError(e.to_string()))?;

        tx.commit()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        Ok(())
    }

    fn query_protocol(
        &self,
        target: &Did,
        protocol: String,
        versions: Vec<dwn_core::message::Version>,
        authorized: bool,
    ) -> Result<Vec<(dwn_core::message::Version, ProtocolDefinition)>, StoreError> {
        let tx = self
            .0
            .r_transaction()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        let mut found = Vec::new();

        for res in tx
            .scan()
            .primary::<Protocol>()
            .map_err(|e| StoreError::BackendError(e.to_string()))?
            .start_with((target.to_string(), protocol))
            .map_err(|e| StoreError::BackendError(e.to_string()))?
        {
            let Ok(prot) = res.as_ref() else {
                warn!("Failed to read protocol during scan");
                continue;
            };

            let def = serde_json::from_slice::<ProtocolDefinition>(&prot.definition)
                .map_err(|e| StoreError::BackendError(e.to_string()))?;

            if !authorized && !def.published {
                continue;
            }

            let version = &prot.version;
            if !versions.is_empty() && !versions.contains(version) {
                continue;
            }

            found.push((version.clone(), def));
        }

        Ok(found)
    }

    fn delete(&self, ds: &dyn DataStore, target: &Did, message: Message) -> Result<(), StoreError> {
        let Descriptor::RecordsDelete(desc) = message.descriptor else {
            panic!("invalid message descriptor: {:?}", message.descriptor)
        };

        debug!("deleting {}", desc.record_id);

        let tx = self
            .0
            .rw_transaction()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        let mut data_cids = Vec::new();

        if let Some(initial_entry) = tx
            .get()
            .primary::<InitialEntry>((target.to_string(), desc.record_id.clone()))
            .map_err(|e| StoreError::BackendError(e.to_string()))?
        {
            let entry: Message = serde_json::from_slice(&initial_entry.entry)
                .map_err(|e| StoreError::BackendError(e.to_string()))?;

            if let Descriptor::RecordsWrite(desc) = entry.descriptor
                && let Some(cid) = &desc.data_cid
            {
                data_cids.push(cid.clone());
            };

            tx.remove(initial_entry)
                .map_err(|e| StoreError::BackendError(e.to_string()))?;
        };

        if let Some(latest_entry) = tx
            .get()
            .primary::<LatestEntry>((target.to_string(), desc.record_id))
            .map_err(|e| StoreError::BackendError(e.to_string()))?
        {
            let entry: Message = serde_json::from_slice(&latest_entry.entry)
                .map_err(|e| StoreError::BackendError(e.to_string()))?;

            if let Descriptor::RecordsWrite(desc) = &entry.descriptor
                && let Some(cid) = &desc.data_cid
            {
                data_cids.push(cid.clone());
            };

            tx.remove(latest_entry)
                .map_err(|e| StoreError::BackendError(e.to_string()))?;
        };

        tx.commit()
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        for cid in data_cids {
            ds.remove_ref(target, &cid)?;
        }

        Ok(())
    }

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

                let entry: Message = serde_json::from_slice(entry)
                    .map_err(|e| StoreError::BackendError(e.to_string()))
                    .unwrap();

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
                let initial_entry: Message = serde_json::from_slice(&initial_entry)
                    .map_err(|e| StoreError::BackendError(e.to_string()))
                    .unwrap();

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

                        let latest_entry: Message = serde_json::from_slice(&latest_entry)
                            .map_err(|e| StoreError::BackendError(e.to_string()))
                            .unwrap();

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

                let entry: Message = serde_json::from_slice(entry)
                    .map_err(|e| StoreError::BackendError(e.to_string()))
                    .unwrap();

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

                if let Some(schema) = filter.schema.as_deref()
                    && desc.schema.as_deref() != Some(schema)
                {
                    return false;
                }

                if let Some(record_id) = filter.record_id.as_deref()
                    && entry.record_id != record_id
                {
                    return false;
                }

                if let Some(parent_id) = filter.parent_id.as_deref() {
                    let Some(context_id) = entry.context_id.as_deref() else {
                        return false;
                    };

                    let Some(context_parent) = context_id.split("/").last() else {
                        return false;
                    };

                    if context_parent != parent_id {
                        return false;
                    }
                }

                if let Some(protocol) = filter.protocol.as_deref()
                    && desc.protocol.as_deref() != Some(protocol)
                {
                    return false;
                }

                if let Some(path) = filter.protocol_path.as_ref()
                    && desc.protocol_path.as_ref() != Some(path)
                {
                    return false;
                }

                if let Some(version) = filter.protocol_version.as_ref() {
                    let Some(desc_version) = &desc.protocol_version else {
                        return false;
                    };

                    let req = VersionReq::parse(&format!("^{version}")).expect("parse version req");

                    if !req.matches(desc_version) {
                        return false;
                    }
                }

                if let Some(data_format) = &filter.data_format
                    && desc.data_format.as_ref() != Some(data_format)
                {
                    return false;
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

        found.sort_by(|a, b| {
            let a: Message = serde_json::from_slice(a)
                .map_err(|e| StoreError::BackendError(e.to_string()))
                .unwrap();
            let b: Message = serde_json::from_slice(b)
                .map_err(|e| StoreError::BackendError(e.to_string()))
                .unwrap();

            match filter.date_sort.unwrap_or_default() {
                DateSort::Ascending => a
                    .descriptor
                    .message_timestamp()
                    .cmp(&b.descriptor.message_timestamp()),
                DateSort::Descending => b
                    .descriptor
                    .message_timestamp()
                    .cmp(&a.descriptor.message_timestamp()),
            }
        });

        let found = found
            .into_iter()
            .map(|x| {
                let x: Message = serde_json::from_slice(&x)
                    .map_err(|e| StoreError::BackendError(e.to_string()))
                    .unwrap();
                x
            })
            .collect();

        Ok(found)
    }

    fn read(
        &self,
        ds: &dyn DataStore,
        target: &Did,
        record_id: &str,
    ) -> Result<Option<Record>, StoreError> {
        debug!("reading {}", record_id);

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

        let initial_entry: Message = serde_json::from_slice(&initial_entry)
            .map_err(|e| StoreError::BackendError(e.to_string()))
            .unwrap();

        let Some(latest_entry) = tx
            .get()
            .primary::<LatestEntry>((target.to_string(), record_id))
            .map_err(|e| StoreError::BackendError(e.to_string()))?
            .map(|v| v.entry)
        else {
            error!("Found initial entry with no latest entry.");
            return Ok(None);
        };

        let mut latest_entry: Message = serde_json::from_slice(&latest_entry)
            .map_err(|e| StoreError::BackendError(e.to_string()))
            .unwrap();

        if let Descriptor::RecordsWrite(desc) = &latest_entry.descriptor
            && let Some(cid) = &desc.data_cid
        {
            latest_entry.data = ds.read(target, cid)?;
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
        debug!("writing {}", message.record_id);

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
                entry: serde_json::to_vec(&message).unwrap(),
            })
            .map_err(|e| StoreError::BackendError(e.to_string()))?;

        if prev.is_none() {
            debug_assert_eq!(
                message.record_id,
                message.descriptor.compute_entry_id().unwrap()
            );

            tx.insert(InitialEntry {
                key: (target.to_string(), message.record_id.clone()),
                entry: serde_json::to_vec(&message).unwrap(),
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
                let prev: Message = serde_json::from_slice(&prev.entry)
                    .map_err(|e| StoreError::BackendError(e.to_string()))
                    .unwrap();

                if let Descriptor::RecordsWrite(desc) = prev.descriptor
                    && let Some(prev_cid) = &desc.data_cid
                {
                    ds.remove_ref(target, prev_cid)?;
                }
            }
        }

        Ok(())
    }
}
