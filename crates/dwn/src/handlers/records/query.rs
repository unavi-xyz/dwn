use dwn_core::{
    message::{Interface, Message, Method},
    store::RecordStore,
};
use tracing::warn;
use xdid::core::did::Did;

use crate::Status;

pub async fn handle(
    records: &dyn RecordStore,
    target: &Did,
    msg: Message,
) -> Result<Vec<Message>, Status> {
    debug_assert_eq!(msg.descriptor.interface, Interface::Records);
    debug_assert_eq!(msg.descriptor.method, Method::Query);

    let authorized = msg.authorization.is_some();

    if let Some(filter) = &msg.descriptor.filter {
        if filter.protocol.is_some() && filter.protocol_version.is_none() {
            return Err(Status {
                code: 400,
                detail: "No protocol version specified",
            });
        }
    }

    let found = records
        .query(
            target,
            &msg.descriptor.filter.unwrap_or_default(),
            authorized,
        )
        .map_err(|e| {
            warn!("Query failed: {:?}", e);
            Status {
                code: 500,
                detail: "Internal error",
            }
        })?;

    Ok(found)
}
