use semver::Version;
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

use crate::message::{
    Message,
    descriptor::{ProtocolDefinition, RecordFilter, RecordsSync},
};

use super::{DataStore, StoreError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Record {
    pub initial_entry: Message,
    pub latest_entry: Message,
}

pub trait RecordStore: Send + Sync {
    fn configure_protocol(&self, target: &Did, message: Message) -> Result<(), StoreError>;

    fn query_protocol(
        &self,
        target: &Did,
        protocol: String,
        versions: Vec<Version>,
        authorized: bool,
    ) -> Result<Vec<(Version, ProtocolDefinition)>, StoreError>;

    fn prepare_sync(&self, target: &Did, authorized: bool) -> Result<RecordsSync, StoreError>;

    fn delete(&self, ds: &dyn DataStore, target: &Did, message: Message) -> Result<(), StoreError>;

    fn query(
        &self,
        target: &Did,
        filter: &RecordFilter,
        authorized: bool,
    ) -> Result<Vec<Message>, StoreError>;

    fn read(
        &self,
        ds: &dyn DataStore,
        target: &Did,
        record_id: &str,
    ) -> Result<Option<Record>, StoreError>;

    fn write(&self, ds: &dyn DataStore, target: &Did, message: Message) -> Result<(), StoreError>;
}
