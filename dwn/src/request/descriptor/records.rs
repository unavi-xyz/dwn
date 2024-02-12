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

    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    pub filter: Option<RecordsQueryFilter>,
}

impl Default for RecordsQuery {
    fn default() -> Self {
        RecordsQuery {
            interface: Interface::Records,
            method: Method::Query,
            message_timestamp: OffsetDateTime::now_utc(),
            filter: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsQueryFilter {
    pub attester: Option<String>,
    pub recepient: Option<String>,
    pub schema: Option<String>,
    #[serde(rename = "recordId")]
    pub record_id: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "contextId")]
    pub context_id: Option<String>,
    #[serde(rename = "dateCreated")]
    pub date_created: Option<FilterDateCreated>,
    pub protocol: Option<String>,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: Option<String>,
    #[serde(rename = "dataFormat")]
    pub data_format: Option<MediaType>,
    #[serde(rename = "dateSort")]
    pub date_sort: Option<FilterDateSort>,
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

    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub protocol: Option<String>,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: Option<String>,
    pub schema: Option<String>,
    pub published: Option<bool>,
    pub encryption: Option<Encryption>,
    #[serde(rename = "dateCreated", with = "time::serde::rfc3339")]
    pub date_created: OffsetDateTime,
    #[serde(rename = "datePublished", with = "time::serde::rfc3339::option")]
    pub date_published: Option<OffsetDateTime>,
}

impl Default for RecordsWrite {
    fn default() -> Self {
        let time = OffsetDateTime::now_utc();

        RecordsWrite {
            interface: Interface::Records,
            method: Method::Write,

            parent_id: None,
            protocol: None,
            protocol_version: None,
            schema: None,
            published: None,
            encryption: None,
            date_created: time,
            date_published: Some(time),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsCommit {
    interface: Interface,
    method: Method,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "dateCreated", with = "time::serde::rfc3339")]
    pub date_created: OffsetDateTime,
}

impl Default for RecordsCommit {
    fn default() -> Self {
        RecordsCommit {
            interface: Interface::Records,
            method: Method::Commit,
            parent_id: "".to_string(),
            date_created: OffsetDateTime::now_utc(),
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
