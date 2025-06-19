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
    pub parent_id: Option<String>,
    pub protocol: Option<String>,
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
    filter: RecordFilter,
}

impl RecordsQueryBuilder {
    pub fn attester(mut self, value: Did) -> Self {
        self.filter.attester = Some(value);
        self
    }

    pub fn recipient(mut self, value: Did) -> Self {
        self.filter.recipient = Some(value);
        self
    }

    pub fn schema(mut self, value: String) -> Self {
        self.filter.schema = Some(value);
        self
    }

    pub fn record_id(mut self, value: String) -> Self {
        self.filter.record_id = Some(value);
        self
    }

    pub fn parent_id(mut self, value: String) -> Self {
        self.filter.parent_id = Some(value);
        self
    }

    pub fn protocol(mut self, value: String, version: semver::Version) -> Self {
        self.filter.protocol = Some(value);
        self.filter.protocol_version = Some(version);
        self
    }

    pub fn data_format(mut self, value: mime::Mime) -> Self {
        self.filter.data_format = Some(value);
        self
    }

    pub fn message_timestamp(mut self, value: DateFilter) -> Self {
        self.filter.date_created = Some(value);
        self
    }

    pub fn date_sort(mut self, value: DateSort) -> Self {
        self.filter.date_sort = Some(value);
        self
    }

    pub fn build(self) -> Result<Message, CidGenerationError> {
        let descriptor = Descriptor::RecordsQuery(Box::new(RecordsQuery {
            interface: Interface::Records,
            method: Method::Query,
            filter: Some(self.filter),
            message_timestamp: OffsetDateTime::now_utc(),
        }));

        Ok(Message {
            record_id: descriptor.compute_entry_id()?,
            data: None,
            descriptor,
            attestation: None,
            authorization: None,
        })
    }
}
