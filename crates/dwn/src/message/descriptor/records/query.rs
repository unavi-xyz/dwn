use iana_media_types::MediaType;
use semver::Version;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::message::descriptor::{Interface, Method};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RecordsQuery {
    interface: Interface,
    method: Method,

    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RecordsFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attester: Option<String>,
    #[serde(rename = "contextId", skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    #[serde(rename = "dataFormat", skip_serializing_if = "Option::is_none")]
    pub data_format: Option<MediaType>,
    #[serde(rename = "dateSort", skip_serializing_if = "Option::is_none")]
    pub date_sort: Option<FilterDateSort>,
    #[serde(rename = "messageTimestamp", skip_serializing_if = "Option::is_none")]
    pub message_timestamp: Option<FilterDateCreated>,
    #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(rename = "protocolVersion", skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<Version>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    #[serde(rename = "recordId", skip_serializing_if = "Option::is_none")]
    pub record_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FilterDateCreated {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    use iana_media_types::Application;

    use super::*;

    #[test]
    fn test_serde() {
        let filter = RecordsFilter {
            attester: Some("did:key:z6Mk".to_string()),
            context_id: Some("https://w3id.org/did/v1".to_string()),
            data_format: Some(MediaType::Application(Application::Json)),
            date_sort: Some(FilterDateSort::CreatedAscending),
            message_timestamp: Some(FilterDateCreated {
                from: Some("2021-01-01T00:00:00Z".to_string()),
                to: Some("2021-12-31T23:59:59Z".to_string()),
            }),
            parent_id: Some("z6Mk".to_string()),
            protocol: Some("https://w3id.org/did/v1".to_string()),
            protocol_version: Some(Version::new(1, 0, 0)),
            recipient: Some("did:key:z6Mk".to_string()),
            record_id: Some("z6Mk".to_string()),
            schema: Some("https://w3id.org/did/v1".to_string()),
        };

        let query = RecordsQuery::new(filter.clone());
        let json = serde_json::to_string(&query).unwrap();
        let query2: RecordsQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(query, query2);
    }
}
