use dwn_core::{
    message::{Interface, Message, Method},
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
    debug_assert_eq!(msg.descriptor.interface, Interface::Records);
    debug_assert_eq!(msg.descriptor.method, Method::Read);

    let authorized = msg.authorization.is_some();

    records
        .read(target, &msg.record_id, authorized)
        .map(|entry| RecordsReadReply { entry })
        .map_err(|e| {
            warn!("Failed to read record {}: {:?}", msg.record_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
