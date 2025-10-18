use dwn_core::{
    message::{Message, descriptor::Descriptor},
    store::RecordStore,
};
use reqwest::StatusCode;
use xdid::core::did::Did;

pub async fn handle(
    _rs: &dyn RecordStore,
    _target: &Did,
    msg: Message,
) -> Result<Vec<Message>, StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::ProtocolsQuery(_)));

    let _authorized = msg.authorization.is_some();

    // TODO

    Ok(Vec::new())
}
