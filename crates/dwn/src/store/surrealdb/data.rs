use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Table, Thing};

use crate::store::{DataStore, DataStoreError, GetDataResults, PutDataResults};

use super::SurrealDB;

const DATA_TABLE_NAME: &str = "data";

impl DataStore for SurrealDB {
    async fn delete(&self, cid: String) -> Result<(), DataStoreError> {
        let db = self.data_db().await.map_err(DataStoreError::BackendError)?;

        let id = Thing::from((Table::from(DATA_TABLE_NAME).to_string(), Id::String(cid)));

        db.delete::<Option<DbData>>(id)
            .await
            .map_err(|e| DataStoreError::BackendError(anyhow::anyhow!(e)))?;

        Ok(())
    }

    async fn get(&self, cid: String) -> Result<Option<GetDataResults>, DataStoreError> {
        let db = self.data_db().await.map_err(DataStoreError::BackendError)?;

        let id = Thing::from((Table::from(DATA_TABLE_NAME).to_string(), Id::String(cid)));

        let res: DbData = match db
            .select(id)
            .await
            .map_err(|e| DataStoreError::BackendError(anyhow::anyhow!(e)))?
        {
            Some(res) => res,
            None => return Ok(None),
        };

        Ok(Some(GetDataResults {
            size: res.data.len(),
            data: res.data,
        }))
    }

    async fn put(&self, cid: String, data: Vec<u8>) -> Result<PutDataResults, DataStoreError> {
        let db = self.data_db().await.map_err(DataStoreError::BackendError)?;

        let id = Thing::from((
            Table::from(DATA_TABLE_NAME).to_string(),
            Id::String(cid.clone()),
        ));

        let size = data.len();

        db.create::<Option<DbData>>(id)
            .content(DbData { cid, data })
            .await
            .map_err(|e| DataStoreError::BackendError(anyhow::anyhow!(e)))?;

        Ok(PutDataResults { size })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DbData {
    cid: String,
    data: Vec<u8>,
}
