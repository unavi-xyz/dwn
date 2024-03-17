use iana_media_types::MediaType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use time::OffsetDateTime;

use super::{Encryption, Interface, Method};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsRead {
    interface: Interface,
    method: Method,

    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    #[serde(rename = "recordId")]
    pub record_id: String,
}

impl RecordsRead {
    pub fn new(record_id: String) -> Self {
        RecordsRead {
            record_id,
            ..Default::default()
        }
    }
}

impl Default for RecordsRead {
    fn default() -> Self {
        RecordsRead {
            interface: Interface::Records,
            method: Method::Read,
            message_timestamp: OffsetDateTime::now_utc(),
            record_id: "".to_string(),
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsQuery {
    interface: Interface,
    method: Method,

    pub filter: Option<Filter>,
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
    pub fn new(filter: Filter) -> Self {
        RecordsQuery {
            filter: Some(filter),
            ..Default::default()
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Filter {
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

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsWrite {
    interface: Interface,
    method: Method,

    pub data_cid: Option<String>,
    #[serde(rename = "datePublished", with = "time::serde::rfc3339::option")]
    pub date_published: Option<OffsetDateTime>,
    pub encryption: Option<Encryption>,
    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub protocol: Option<String>,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: Option<String>,
    pub published: Option<bool>,
    pub schema: Option<String>,
}

impl Default for RecordsWrite {
    fn default() -> Self {
        let time = OffsetDateTime::now_utc();

        RecordsWrite {
            interface: Interface::Records,
            method: Method::Write,

            data_cid: None,
            date_published: Some(time),
            encryption: None,
            message_timestamp: time,
            parent_id: None,
            protocol: None,
            protocol_version: None,
            published: None,
            schema: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsCommit {
    interface: Interface,
    method: Method,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
}

impl RecordsCommit {
    pub fn new(parent_id: String) -> Self {
        RecordsCommit {
            parent_id,
            ..Default::default()
        }
    }
}

impl Default for RecordsCommit {
    fn default() -> Self {
        RecordsCommit {
            interface: Interface::Records,
            method: Method::Commit,
            parent_id: "".to_string(),
            message_timestamp: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsDelete {
    interface: Interface,
    method: Method,
    #[serde(rename = "recordId")]
    pub record_id: String,
    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
}

impl Default for RecordsDelete {
    fn default() -> Self {
        RecordsDelete {
            interface: Interface::Records,
            method: Method::Delete,
            record_id: "".to_string(),
            message_timestamp: OffsetDateTime::now_utc(),
        }
    }
}
