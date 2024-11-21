use xdid::core::did::Did;

use crate::message::data::Data;

use super::StoreError;

/// Maps IPLD hashes to their contents.
pub trait DataStore: Send + Sync {
    fn read(&self, target: &Did, cid: &str) -> Result<Option<Data>, StoreError>;

    /// Adds a reference to a CID.
    fn add_ref(&self, target: &Did, cid: &str, data: Option<Data>) -> Result<(), StoreError>;

    /// Removes a reference to a CID.
    fn remove_ref(&self, target: &Did, cid: &str) -> Result<(), StoreError>;
}
