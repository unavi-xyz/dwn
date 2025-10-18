use dwn_core::{
    message::{Message, descriptor::Descriptor},
    store::RecordStore,
};
use reqwest::StatusCode;
use tracing::warn;
use xdid::core::did::Did;

pub async fn handle(rs: &dyn RecordStore, target: &Did, msg: Message) -> Result<(), StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::ProtocolsConfigure(_)));

    if msg.authorization.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    rs.configure_protocol(target, msg).map_err(|e| {
        warn!("Protocol configure failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}
