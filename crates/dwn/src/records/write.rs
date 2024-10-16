use dwn_core::{
    message::{Interface, Message, Method},
    store::RecordStore,
};

use crate::Status;

pub fn handle(records: &dyn RecordStore, msg: Message) -> Result<(), Status> {
    debug_assert_eq!(msg.descriptor.interface, Interface::Records);
    debug_assert_eq!(msg.descriptor.method, Method::Write);

    let did = "did:example:123";

    let Ok(prev) = records.read(did, &msg.record_id) else {
        return Err(Status {
            code: 500,
            detail: "Internal error.",
        });
    };

    match prev {
        Some(_) => {
            todo!("overwrite record");
        }
        None => {
            // For new records, the record ID must match the message.
            match msg.descriptor.compute_record_id() {
                Ok(id) => {
                    if id != msg.record_id {
                        return Err(Status {
                            code: 400,
                            detail: "Invalid record ID.",
                        });
                    }
                }
                Err(_) => {
                    return Err(Status {
                        code: 400,
                        detail: "Failed to compute record ID for message.",
                    })
                }
            };

            if records.write(did, msg).is_err() {
                return Err(Status {
                    code: 500,
                    detail: "Internal error.",
                });
            };
        }
    }

    Ok(())
}
