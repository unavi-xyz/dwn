use dwn_core::message::{
    cid::CidGenerationError, mime::Mime, DateFilter, DateSort, Descriptor, Filter, Interface,
    Message, Method, OffsetDateTime, Version,
};
use xdid::core::did::Did;

#[derive(Default)]
pub struct RecordsQueryBuilder {
    filter: Filter,
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

    pub fn context_id(mut self, value: String) -> Self {
        self.filter.context_id = Some(value);
        self
    }

    pub fn protocol(mut self, value: String, version: Version) -> Self {
        self.filter.protocol = Some(value);
        self.filter.protocol_version = Some(version);
        self
    }

    pub fn data_format(mut self, value: Mime) -> Self {
        self.filter.data_format = Some(value);
        self
    }

    pub fn date_created(mut self, value: DateFilter) -> Self {
        self.filter.date_created = Some(value);
        self
    }

    pub fn date_sort(mut self, value: DateSort) -> Self {
        self.filter.date_sort = Some(value);
        self
    }

    pub fn build(self) -> Result<Message, CidGenerationError> {
        let descriptor = Descriptor {
            interface: Interface::Records,
            method: Method::Query,
            filter: Some(self.filter),
            data_cid: None,
            data_format: None,
            parent_id: None,
            protocol: None,
            protocol_version: None,
            published: None,
            schema: None,
            date_created: OffsetDateTime::now_utc(),
        };

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
