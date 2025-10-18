use dwn_core::message::descriptor::Descriptor;
use reqwest::StatusCode;
use tracing::warn;

use crate::ProcessContext;

pub async fn handle(
    ProcessContext {
        rs,
        ds,
        validation,
        target,
        msg,
    }: ProcessContext<'_>,
) -> Result<(), StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::RecordsDelete(_)));

    if !validation.authenticated.contains(target) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    rs.delete(ds, target, msg).map_err(|e| {
        warn!("Failed to delete record: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}
