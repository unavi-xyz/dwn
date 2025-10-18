use dwn_core::message::descriptor::Descriptor;
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
) -> Result<(), StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::ProtocolsConfigure(_)));

    if !validation.authenticated.contains(target) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    rs.configure_protocol(target, msg).map_err(|e| {
        warn!("Protocol configure failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}
