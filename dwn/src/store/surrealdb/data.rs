use libipld::Cid;
use thiserror::Error;

use crate::store::{DataStore, GetDataResults, PutDataResults};

use super::SurrealDB;

#[derive(Error, Debug)]
pub enum DataStoreError {}

impl DataStore for SurrealDB {
    type Error = DataStoreError;

    async fn delete(&self, tenant: &str, record_id: &str, cid: Cid) -> Result<(), Self::Error> {
        unimplemented!()
    }
    async fn get<T: std::io::prelude::Read + Send + Sync>(
        &self,
        tenant: &str,
        record_id: &str,
        cid: Cid,
    ) -> Result<Option<GetDataResults<T>>, Self::Error> {
        unimplemented!()
    }
    async fn put(
        &self,
        tenant: &str,
        record_id: &str,
        cid: Cid,
        value: impl std::io::prelude::Write + Send + Sync,
    ) -> Result<PutDataResults, Self::Error> {
        unimplemented!()
    }
}
