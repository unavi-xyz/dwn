use dwn_core::{
    message::{Interface, Message, Method},
    reply::RecordsQueryReply,
    store::RecordStore,
};
use reqwest::StatusCode;
use tracing::{debug, warn};
use xdid::core::did::Did;

pub async fn handle(
    records: &dyn RecordStore,
    target: &Did,
    msg: Message,
) -> Result<RecordsQueryReply, StatusCode> {
    debug_assert_eq!(msg.descriptor.interface, Interface::Records);
    debug_assert_eq!(msg.descriptor.method, Method::Query);

    let authorized = msg.authorization.is_some();

    if let Some(filter) = &msg.descriptor.filter {
        if filter.protocol.is_some() && filter.protocol_version.is_none() {
            debug!("No protocol version specified");
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    records
        .query(
            target,
            &msg.descriptor.filter.unwrap_or_default(),
            authorized,
        )
        .map(|entries| RecordsQueryReply { entries })
        .map_err(|e| {
            warn!("Query failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
