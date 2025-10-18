use dwn_core::{message::descriptor::Descriptor, reply::RecordsQueryReply};
use reqwest::StatusCode;
use tracing::{debug, warn};

use crate::ProcessContext;

pub async fn handle(
    ProcessContext {
        rs,
        validation,
        target,
        msg,
        ..
    }: ProcessContext<'_>,
) -> Result<RecordsQueryReply, StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::RecordsQuery(_)));

    let Descriptor::RecordsQuery(desc) = msg.descriptor else {
        panic!("invalid descriptor: {:?}", msg.descriptor);
    };

    if let Some(filter) = &desc.filter
        && filter.protocol.is_some()
        && filter.protocol_version.is_none()
    {
        debug!("No protocol version specified");
        return Err(StatusCode::BAD_REQUEST);
    }

    let authorized = validation.authenticated.contains(target);

    rs.query(target, &desc.filter.unwrap_or_default(), authorized)
        .map(|entries| RecordsQueryReply { entries })
        .map_err(|e| {
            warn!("Query failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
