use dwn_core::{
    message::{Message, descriptor::Descriptor},
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
) -> Result<(), StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::RecordsDelete(_)));

    if msg.authorization.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    rs.delete(ds, target, msg).map_err(|e| {
        warn!("Failed to delete record: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}
