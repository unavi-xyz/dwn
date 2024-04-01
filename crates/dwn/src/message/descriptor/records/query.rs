use iana_media_types::MediaType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use time::OffsetDateTime;

use crate::message::descriptor::{Interface, Method};

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsQuery {
    interface: Interface,
    method: Method,

    pub filter: Option<RecordsFilter>,
    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
}

impl Default for RecordsQuery {
    fn default() -> Self {
        RecordsQuery {
            interface: Interface::Records,
            method: Method::Query,

            filter: None,
            message_timestamp: OffsetDateTime::now_utc(),
        }
    }
}

impl RecordsQuery {
    pub fn new(filter: RecordsFilter) -> Self {
        RecordsQuery {
            filter: Some(filter),
            ..Default::default()
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RecordsFilter {
    pub attester: Option<String>,
    #[serde(rename = "contextId")]
    pub context_id: Option<String>,
    #[serde(rename = "dataFormat")]
    pub data_format: Option<MediaType>,
    #[serde(rename = "dateSort")]
    pub date_sort: Option<FilterDateSort>,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: Option<FilterDateCreated>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub protocol: Option<String>,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: Option<String>,
    pub recipient: Option<String>,
    #[serde(rename = "recordId")]
    pub record_id: Option<String>,
    pub schema: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FilterDateCreated {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum FilterDateSort {
    #[serde(rename = "createdAscending")]
    CreatedAscending,
    #[serde(rename = "createdDescending")]
    CreatedDescending,
    #[serde(rename = "publishedAscending")]
    PublishedAscending,
    #[serde(rename = "publishedDescending")]
    PublishedDescending,
}
