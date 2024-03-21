use std::collections::HashMap;

use libipld::Cid;

use crate::{
    handlers::{RecordsReadReply, Reply, Status},
    message::{
        descriptor::{Descriptor, Filter, FilterDateSort},
        Message, RawMessage, ValidatedMessage,
    },
    store::{DataStore, MessageStore},
    HandleMessageError,
};

use super::util::create_entry_id_map;

pub async fn handle_records_read(
    data_store: &impl DataStore,
    message_store: &impl MessageStore,
    message: ValidatedMessage,
) -> Result<Reply, HandleMessageError> {
    let tenant = message.tenant();

    let descriptor = match &message.read().descriptor {
        Descriptor::RecordsRead(descriptor) => descriptor,
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a RecordsRead message".to_string(),
            ));
        }
    };

    let messages = message_store
        .query(
            tenant,
            Filter {
                record_id: Some(descriptor.record_id.clone()),
                date_sort: Some(FilterDateSort::CreatedDescending),
                ..Default::default()
            },
        )
        .await?;

    let latest_checkpoint = messages
        .iter()
        .find(|m| matches!(m.descriptor, Descriptor::RecordsDelete(_)))
        .or(messages.last())
        .ok_or(HandleMessageError::InvalidDescriptor(
            "Record not found".to_string(),
        ))?;

    // Get the record that has the data.
    // TODO: Simplify this, without RecordsCommit this is not needed
    let entry_id_map = create_entry_id_map(&messages)?;
    let record_entry_id = latest_checkpoint.generate_record_id()?;
    let data_cid = get_data_cid(&record_entry_id, &entry_id_map);

    let data = match data_cid {
        Some(data_cid) => {
            let data_cid = Cid::try_from(data_cid).map_err(|e| {
                HandleMessageError::InvalidDescriptor(format!("Invalid data CID: {}", e))
            })?;
            let res = data_store.get(data_cid.to_string()).await?;
            res.map(|res| res.data)
        }
        None => None,
    };

    Ok(RecordsReadReply {
        data,
        record: latest_checkpoint.to_owned(),
        status: Status::ok(),
    }
    .into())
}

/// Get the data CID for a given entry ID.
/// This will search up the chain of parent messages until it finds a RecordsWrite message.
fn get_data_cid(entry_id: &str, messages: &HashMap<String, &RawMessage>) -> Option<String> {
    let entry = messages.get(entry_id)?;

    match &entry.descriptor {
        Descriptor::RecordsWrite(desc) => desc.data_cid.clone(),
        _ => None,
    }
}
