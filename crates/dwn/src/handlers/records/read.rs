use dwn_core::{
    message::{Interface, Message, Method},
    store::RecordStore,
};
use xdid::core::did::Did;

use crate::Status;

pub fn handle(
    records: &dyn RecordStore,
    target: &Did,
    msg: Message,
) -> Result<Option<Message>, Status> {
    debug_assert_eq!(msg.descriptor.interface, Interface::Records);
    debug_assert_eq!(msg.descriptor.method, Method::Read);

    let authorized = msg.authorization.is_some();

    let Ok(found) = records.read(target, &msg.record_id, authorized) else {
        return Err(Status {
            code: 500,
            detail: "Internal error",
        });
    };

    Ok(found)
}
