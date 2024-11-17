use dwn_core::{
    message::{descriptor::Descriptor, Message},
    reply::RecordsReadReply,
    store::RecordStore,
};
use reqwest::StatusCode;
use tracing::warn;
use xdid::core::did::Did;

pub fn handle(
    records: &dyn RecordStore,
    target: &Did,
    msg: Message,
) -> Result<RecordsReadReply, StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::RecordsRead(_)));

    let Descriptor::RecordsRead(desc) = msg.descriptor else {
        panic!("invalid descriptor: {:?}", msg.descriptor);
    };

    let record = records.read(target, &desc.record_id).map_err(|e| {
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
