use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use surrealdb::Connection;

use crate::store::{DataStore, DataStoreError, PutDataResults, StoredData};

use super::SurrealStore;

const DATA_TABLE: &str = "data";

impl<T: Connection> DataStore for SurrealStore<T> {
    async fn delete(&self, cid: &str) -> Result<(), DataStoreError> {
        let db = self.data_db().await.map_err(DataStoreError::BackendError)?;

        db.delete::<Option<DbData>>((DATA_TABLE, cid))
            .await
            .map_err(|e| DataStoreError::BackendError(anyhow!(e)))?;

        Ok(())
    }

    async fn get(&self, cid: &str) -> Result<Option<StoredData>, DataStoreError> {
        let db = self.data_db().await.map_err(DataStoreError::BackendError)?;

        let res: Result<Option<DbData>, _> = db.select((DATA_TABLE, cid)).await;

        res.map(|r| r.map(|r| r.data))
            .map_err(|e| DataStoreError::BackendError(anyhow!(e)))
    }

    async fn put(&self, cid: String, data: StoredData) -> Result<PutDataResults, DataStoreError> {
        let db = self.data_db().await.map_err(DataStoreError::BackendError)?;

        let size = data.len();

        db.create::<Option<DbData>>((DATA_TABLE, &cid))
            .content(DbData { cid, data })
            .await
            .map_err(|e| DataStoreError::BackendError(anyhow!(e)))?;

        Ok(PutDataResults { size })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DbData {
    cid: String,
    data: StoredData,
}
