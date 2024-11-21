use dwn_core::{
    message::{descriptor::Descriptor, Message},
    reply::RecordsQueryReply,
    store::RecordStore,
};
use reqwest::StatusCode;
use tracing::{debug, warn};
use xdid::core::did::Did;

pub async fn handle(
    rs: &dyn RecordStore,
    target: &Did,
    msg: Message,
) -> Result<RecordsQueryReply, StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::RecordsQuery(_)));

    let authorized = msg.authorization.is_some();

    let Descriptor::RecordsQuery(desc) = msg.descriptor else {
        panic!("invalid descriptor: {:?}", msg.descriptor);
    };

    if let Some(filter) = &desc.filter {
        if filter.protocol.is_some() && filter.protocol_version.is_none() {
            debug!("No protocol version specified");
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    rs.query(target, &desc.filter.unwrap_or_default(), authorized)
        .map(|entries| RecordsQueryReply { entries })
        .map_err(|e| {
            warn!("Query failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
