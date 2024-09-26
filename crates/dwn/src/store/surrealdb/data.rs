use std::{future::Future, pin::Pin};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use surrealdb::Connection;

use crate::store::{DataStore, DataStoreError, PutDataResults, StoredData};

use super::SurrealStore;

const DATA_TABLE: &str = "data";

impl<T: Connection> DataStore for SurrealStore<T> {
    fn delete(
        &self,
        cid: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), DataStoreError>> + Send + Sync>> {
        let store = self.clone();

        Box::pin(async move {
            let db = store
                .data_db()
                .await
                .map_err(DataStoreError::BackendError)?;

            db.delete::<Option<DbData>>((DATA_TABLE, cid))
                .await
                .map_err(|e| DataStoreError::BackendError(anyhow!(e)))?;

            Ok(())
        })
    }

    fn get(
        &self,
        cid: String,
    ) -> Pin<Box<dyn Future<Output = Result<Option<StoredData>, DataStoreError>> + Send + Sync>>
    {
        let store = self.clone();

        Box::pin(async move {
            let db = store
                .data_db()
                .await
                .map_err(DataStoreError::BackendError)?;

            let res: Result<Option<DbData>, _> = db.select((DATA_TABLE, cid)).await;

            res.map(|r| r.map(|r| r.data))
                .map_err(|e| DataStoreError::BackendError(anyhow!(e)))
        })
    }

    fn put(
        &self,
        cid: String,
        data: StoredData,
    ) -> Pin<Box<dyn Future<Output = Result<PutDataResults, DataStoreError>> + Send + Sync>> {
        let store = self.clone();

        Box::pin(async move {
            let db = store
                .data_db()
                .await
                .map_err(DataStoreError::BackendError)?;

            let size = data.len();

            db.create::<Option<DbData>>((DATA_TABLE, &cid))
                .content(DbData { cid, data })
                .await
                .map_err(|e| DataStoreError::BackendError(anyhow!(e)))?;

            Ok(PutDataResults { size })
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DbData {
    cid: String,
    data: StoredData,
}
