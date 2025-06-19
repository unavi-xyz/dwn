use dwn_core::{
    message::{Message, descriptor::Descriptor},
    reply::RecordsSyncReply,
    store::{DataStore, RecordStore, StoreError},
};
use reqwest::StatusCode;
use tracing::warn;
use xdid::core::did::Did;

pub fn handle(
    ds: &dyn DataStore,
    rs: &dyn RecordStore,
    target: &Did,
    msg: Message,
) -> Result<RecordsSyncReply, StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::RecordsSync(_)));

    let Descriptor::RecordsSync(desc) = msg.descriptor else {
        panic!("invalid descriptor: {:?}", msg.descriptor);
    };

    let authorized = msg.authorization.is_some();

    let mut reply = RecordsSyncReply {
        conflict: Vec::new(),
        local_only: Vec::new(),
        remote_only: Vec::new(),
    };

    let mut local = rs.prepare_sync(target, authorized).map_err(|e| {
        warn!("Failed to prepare sync {}: {:?}", msg.record_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    for record in desc.local_records {
        // Remove from local records.
        if let Some(found_idx) = local
            .local_records
            .iter()
            .position(|v| v.record_id == record.record_id)
        {
            local.local_records.remove(found_idx);
        };

        // Process given record.
        if let Some(found) = rs.read(ds, target, &record.record_id).map_err(|e| {
            warn!("Failed to read record {}: {:?}", msg.record_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })? {
            if found
                .latest_entry
                .descriptor
                .compute_entry_id()
                .map_err(|e| {
                    warn!("Failed to compute entry id {}: {:?}", msg.record_id, e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                != record.latest_entry_id
            {
                reply.conflict.push(found.latest_entry);
            }
        } else {
            reply.local_only.push(record.record_id);
        };
    }

    reply.remote_only = local
        .local_records
        .into_iter()
        .map(|id| match rs.read(ds, target, &id.record_id) {
            Ok(Some(r)) => Ok(r),
            Ok(None) => Err(StoreError::BackendError(
                "Sync record not found".to_string(),
            )),
            Err(e) => Err(e),
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            warn!(
                "Failed to read record {} during sync: {:?}",
                msg.record_id, e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(reply)
}
