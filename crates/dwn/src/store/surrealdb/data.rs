use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use surrealdb::{
    sql::{Id, Table, Thing},
    Connection,
};

use crate::store::{DataStore, DataStoreError, PutDataResults, StoredData};

use super::SurrealStore;

const DATA_TABLE_NAME: &str = "data";

impl<T: Connection> DataStore for SurrealStore<T> {
    async fn delete(&self, cid: String) -> Result<(), DataStoreError> {
        let db = self.data_db().await.map_err(DataStoreError::BackendError)?;

        let id = Thing::from((Table::from(DATA_TABLE_NAME).to_string(), Id::String(cid)));

        db.delete::<Option<DbData>>(id)
            .await
            .map_err(|e| DataStoreError::BackendError(anyhow!(e)))?;

        Ok(())
    }

    async fn get(&self, cid: String) -> Result<Option<StoredData>, DataStoreError> {
        let db = self.data_db().await.map_err(DataStoreError::BackendError)?;

        let id = Thing::from((Table::from(DATA_TABLE_NAME).to_string(), Id::String(cid)));

        let res: Result<Option<DbData>, _> = db.select(id).await;

        res.map(|r| r.map(|r| r.data))
            .map_err(|e| DataStoreError::BackendError(anyhow!(e)))
    }

    async fn put(&self, cid: String, data: StoredData) -> Result<PutDataResults, DataStoreError> {
        let db = self.data_db().await.map_err(DataStoreError::BackendError)?;

        let id = Thing::from((
            Table::from(DATA_TABLE_NAME).to_string(),
            Id::String(cid.clone()),
        ));

        let size = data.len();

        db.create::<Option<DbData>>(id)
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
