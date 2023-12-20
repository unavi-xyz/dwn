use iana_media_types::MediaType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use time::OffsetDateTime;

use super::message::{CommitStrategy, Descriptor, Encryption, Interface, Method};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RecordsRead {
    descriptor: RecordsReadDescriptor,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsReadDescriptor {
    interface: Interface,
    method: Method,
    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    #[serde(rename = "recordId")]
    pub record_id: String,
}

impl Descriptor for RecordsReadDescriptor {}

impl Default for RecordsReadDescriptor {
    fn default() -> Self {
        RecordsReadDescriptor {
            interface: Interface::Records,
            method: Method::Read,
            message_timestamp: OffsetDateTime::now_utc(),
            record_id: "".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RecordsQuery {
    descriptor: RecordsQueryDescriptor,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsQueryDescriptor {
    interface: Interface,
    method: Method,
    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
    pub filter: Option<RecordsQueryFilter>,
}

impl Descriptor for RecordsQueryDescriptor {}

impl Default for RecordsQueryDescriptor {
    fn default() -> Self {
        RecordsQueryDescriptor {
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsWrite {
    #[serde(rename = "recordId")]
    pub record_id: String,
    pub descriptor: RecordsWriteDescriptor,
}

impl Default for RecordsWrite {
    fn default() -> Self {
        let descriptor = RecordsWriteDescriptor::default();

        RecordsWrite {
            record_id: descriptor.record_id().unwrap(),
            descriptor,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsWriteDescriptor {
    interface: Interface,
    method: Method,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub protocol: Option<String>,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: Option<String>,
    pub schema: Option<String>,
    #[serde(rename = "commitStrategy")]
    pub commit_strategy: Option<CommitStrategy>,
    pub published: Option<bool>,
    pub encryption: Option<Encryption>,
    #[serde(rename = "dateCreated", with = "time::serde::rfc3339")]
    pub date_created: OffsetDateTime,
    #[serde(rename = "datePublished", with = "time::serde::rfc3339::option")]
    pub date_published: Option<OffsetDateTime>,
}

impl Descriptor for RecordsWriteDescriptor {}

impl Default for RecordsWriteDescriptor {
    fn default() -> Self {
        let time = OffsetDateTime::now_utc();

        RecordsWriteDescriptor {
            interface: Interface::Records,
            method: Method::Write,
            parent_id: None,
            protocol: None,
            protocol_version: None,
            schema: None,
            commit_strategy: None,
            published: None,
            encryption: None,
            date_created: time,
            date_published: Some(time),
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RecordsCommit {
    #[serde(rename = "recordId")]
    pub record_id: String,
    #[serde(rename = "contextId")]
    pub context_id: Option<String>,
    pub descriptor: RecordsCommitDescriptor,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsCommitDescriptor {
    interface: Interface,
    method: Method,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "commitStrategy")]
    pub commit_strategy: CommitStrategy,
    #[serde(rename = "dateCreated", with = "time::serde::rfc3339")]
    pub date_created: OffsetDateTime,
}

impl Descriptor for RecordsCommitDescriptor {}

impl Default for RecordsCommitDescriptor {
    fn default() -> Self {
        RecordsCommitDescriptor {
            interface: Interface::Records,
            method: Method::Commit,
            parent_id: "".to_string(),
            commit_strategy: CommitStrategy::JsonPatch,
            date_created: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RecordsDelete {
    pub descriptor: RecordsDeleteDescriptor,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsDeleteDescriptor {
    interface: Interface,
    method: Method,
    #[serde(rename = "recordId")]
    pub record_id: String,
    #[serde(rename = "messageTimestamp", with = "time::serde::rfc3339")]
    pub message_timestamp: OffsetDateTime,
}

impl Descriptor for RecordsDeleteDescriptor {}

impl Default for RecordsDeleteDescriptor {
    fn default() -> Self {
        RecordsDeleteDescriptor {
            interface: Interface::Records,
            method: Method::Delete,
            record_id: "".to_string(),
            message_timestamp: OffsetDateTime::now_utc(),
        }
    }
}
