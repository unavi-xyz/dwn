use libipld::Cid;

use crate::{
    message::{
        descriptor::{
            records::{FilterDateSort, RecordsFilter},
            Descriptor,
        },
        DwnRequest,
    },
    reply::{MessageReply, RecordsReadReply, Status},
    store::{DataStore, MessageStore},
    HandleMessageError,
};

pub async fn handle_records_read(
    data_store: &impl DataStore,
    message_store: &impl MessageStore,
    DwnRequest { target, message }: DwnRequest,
) -> Result<MessageReply, HandleMessageError> {
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
        .query_records(
            target.clone(),
            message.author().as_deref(),
            authorized,
            RecordsFilter {
                record_id: Some(descriptor.record_id.clone()),
                date_sort: Some(FilterDateSort::CreatedDescending),
                ..Default::default()
            },
        )
        .await?;

    let mut latest = messages
        .iter()
        .find(|m| {
            matches!(
                m.descriptor,
                Descriptor::RecordsDelete(_) | Descriptor::RecordsWrite(_)
            )
        })
        .or(messages.last())
        .ok_or(HandleMessageError::InvalidDescriptor(
            "Record not found".to_string(),
        ))?
        .to_owned();

    // Read data.
    latest.data = match &latest.descriptor {
        Descriptor::RecordsWrite(descriptor) => {
            if let Some(data_cid) = &descriptor.data_cid {
                let data_cid = Cid::try_from(data_cid.as_str()).map_err(|e| {
                    HandleMessageError::InvalidDescriptor(format!("Invalid data CID: {}", e))
                })?;
                let res = data_store.get(data_cid.to_string()).await?;
                res.map(|res| res.into())
            } else {
                None
            }
        }
        _ => None,
    };

    Ok(RecordsReadReply {
        record: Box::new(latest),
        status: Status::ok(),
    }
    .into())
}
