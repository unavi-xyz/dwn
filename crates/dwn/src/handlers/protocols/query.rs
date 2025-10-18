use dwn_core::message::{Message, descriptor::Descriptor};
use reqwest::StatusCode;

use crate::ProcessContext;

pub async fn handle(
    ProcessContext { msg, .. }: ProcessContext<'_>,
) -> Result<Vec<Message>, StatusCode> {
    debug_assert!(matches!(msg.descriptor, Descriptor::ProtocolsQuery(_)));

    // TODO

    Ok(Vec::new())
}
