use std::io::{Read, Write};

use libipld::Cid;
use thiserror::Error;

use crate::store::{DataStore, GetDataResults, PutDataResults};

use super::SurrealDB;

#[derive(Error, Debug)]
pub enum DataStoreError {}

impl DataStore for SurrealDB {
    type Error = DataStoreError;

    async fn delete(&self, _tenant: &str, _record_id: &str, _cid: Cid) -> Result<(), Self::Error> {
        unimplemented!()
    }
    async fn get<T: Read + Send + Sync>(
        &self,
        _tenant: &str,
        _record_id: &str,
        _cid: Cid,
    ) -> Result<Option<GetDataResults<T>>, Self::Error> {
        unimplemented!()
    }
    async fn put(
        &self,
        _tenant: &str,
        _record_id: &str,
        _cid: Cid,
        _value: impl Write + Send + Sync,
    ) -> Result<PutDataResults, Self::Error> {
        unimplemented!()
    }
}
