use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as, skip_serializing_none};
use time::OffsetDateTime;
use xdid::core::did::Did;

use crate::message::{
    Message,
    cid::CidGenerationError,
    descriptor::{Descriptor, Interface, Method},
};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordsQuery {
    interface: Interface,
    method: Method,
    pub filter: Option<RecordFilter>,
    #[serde(with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordFilter {
    pub attester: Option<Did>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub data_format: Option<mime::Mime>,
    pub date_created: Option<DateFilter>,
    pub date_sort: Option<DateSort>,
    pub protocol: Option<String>,
    pub protocol_path: Option<String>,
    pub protocol_version: Option<semver::Version>,
    pub recipient: Option<Did>,
    pub record_id: Option<String>,
    pub schema: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateFilter {
    #[serde(with = "time::serde::rfc3339")]
    pub from: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub to: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DateSort {
    Ascending,
    #[default]
    Descending,
}

#[derive(Default)]
pub struct RecordsQueryBuilder {
    pub filter: RecordFilter,
}

impl RecordsQueryBuilder {
    pub fn build(self) -> Result<Message, CidGenerationError> {
        let descriptor = Descriptor::RecordsQuery(Box::new(RecordsQuery {
            interface: Interface::Records,
            method: Method::Query,
            filter: Some(self.filter),
            message_timestamp: OffsetDateTime::now_utc(),
        }));

        Ok(Message {
            record_id: descriptor.compute_entry_id()?,
            context_id: None,
            data: None,
            descriptor,
            attestation: None,
            authorization: None,
        })
    }
}
