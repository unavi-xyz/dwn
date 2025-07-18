use dwn_core::{
    message::{Message, descriptor::Descriptor},
    reply::RecordsReadReply,
    store::{DataStore, RecordStore},
};
use reqwest::StatusCode;
use tracing::warn;
use xdid::core::did::Did;

pub fn handle(
    ds: &dyn DataStore,
    rs: &dyn RecordStore,
    target: &Did,
    msg: Message,
) -> Result<RecordsReadReply, StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::RecordsRead(_)));

    let Descriptor::RecordsRead(desc) = msg.descriptor else {
        panic!("invalid descriptor: {:?}", msg.descriptor);
    };

    let record = rs.read(ds, target, &desc.record_id).map_err(|e| {
        warn!("Failed to read record {}: {:?}", msg.record_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let authorized = msg.authorization.is_some();

    Ok(RecordsReadReply {
        entry: record.map(|r| r.latest_entry).and_then(|m| {
            if let Descriptor::RecordsWrite(d) = &m.descriptor {
                if d.published != Some(true) && !authorized {
                    None
                } else {
                    Some(m)
                }
            } else {
                None
            }
        }),
    })
}
