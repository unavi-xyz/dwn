use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

use crate::message::{
    Message,
    descriptor::{RecordFilter, RecordsSync},
};

use super::{DataStore, StoreError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Record {
    pub initial_entry: Message,
    pub latest_entry: Message,
}

pub trait RecordStore: Send + Sync {
    /// Prepares a RecordsSync message to be sent to a remote DWN.
    fn prepare_sync(&self, target: &Did, authorized: bool) -> Result<RecordsSync, StoreError>;

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
