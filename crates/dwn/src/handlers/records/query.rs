use dwn_core::{message::descriptor::Descriptor, reply::RecordsQueryReply};
use reqwest::StatusCode;
use tracing::warn;

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

    let authorized = validation.authenticated.contains(target);

    rs.query(target, &desc.filter.unwrap_or_default(), authorized)
        .map(|entries| RecordsQueryReply { entries })
        .map_err(|e| {
            warn!("Query failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
