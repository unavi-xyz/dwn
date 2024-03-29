use std::collections::HashMap;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use libipld::Cid;

use crate::{
    handlers::{RecordsReadReply, Reply, Status},
    message::{
        descriptor::Descriptor, Data, EncryptedData, Filter, FilterDateSort, Message, Request,
    },
    store::{DataStore, MessageStore},
    HandleMessageError,
};

use super::util::create_entry_id_map;

pub async fn handle_records_read(
    data_store: &impl DataStore,
    message_store: &impl MessageStore,
    Request { target, message }: Request,
) -> Result<Reply, HandleMessageError> {
    let authorized = message.is_authorized(&target).await;

    let descriptor = match &message.descriptor {
        Descriptor::RecordsRead(descriptor) => descriptor,
        _ => {
            return Err(HandleMessageError::InvalidDescriptor(
                "Not a RecordsRead message".to_string(),
            ));
        }
    };

    let messages = message_store
        .query(
            target.clone(),
            authorized,
            Filter {
                record_id: Some(descriptor.record_id.clone()),
                date_sort: Some(FilterDateSort::CreatedDescending),
                ..Default::default()
            },
        )
        .await?;

    let mut record = messages
        .iter()
        .find(|m| matches!(m.descriptor, Descriptor::RecordsDelete(_)))
        .or(messages.last())
        .ok_or(HandleMessageError::InvalidDescriptor(
            "Record not found".to_string(),
        ))?
        .to_owned();

    // Read data.
    // TODO: Simplify this, without RecordsCommit this is not needed
    let entry_id_map = create_entry_id_map(&messages)?;
    let record_entry_id = record.entry_id()?;
    let data_cid = get_data_cid(&record_entry_id, &entry_id_map);

    let data_bytes = match data_cid {
        Some(data_cid) => {
            let data_cid = Cid::try_from(data_cid).map_err(|e| {
                HandleMessageError::InvalidDescriptor(format!("Invalid data CID: {}", e))
            })?;
            let res = data_store.get(data_cid.to_string()).await?;
            res.map(|res| res.data)
        }
        None => None,
    };

    if let Some(bytes) = data_bytes {
        match &record.data {
            Some(Data::Base64(_)) => {
                record.data = Some(Data::new_base64(&bytes));
            }
            Some(Data::Encrypted(data)) => {
                record.data = Some(Data::Encrypted(EncryptedData {
                    ciphertext: URL_SAFE_NO_PAD.encode(&bytes),
                    iv: data.iv.clone(),
                    protected: data.protected.clone(),
                    recipients: data.recipients.clone(),
                    tag: data.tag.clone(),
                }))
            }
            None => {}
        }
    }

    Ok(RecordsReadReply {
        record: Box::new(record),
        status: Status::ok(),
    }
    .into())
}

/// Get the data CID for a given entry ID.
/// This will search up the chain of parent messages until it finds a RecordsWrite message.
fn get_data_cid(entry_id: &str, messages: &HashMap<String, &Message>) -> Option<String> {
    let entry = messages.get(entry_id)?;

    match &entry.descriptor {
        Descriptor::RecordsWrite(desc) => desc.data_cid.clone(),
        _ => None,
    }
}
