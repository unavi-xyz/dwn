use libipld::Cid;
use surrealdb::sql::{Id, Table, Thing};
use thiserror::Error;

use crate::store::{DataStore, GetDataResults, PutDataResults};

use super::{
    model::{CreateData, GetData},
    SurrealDB,
};

const CID_TABLE_NAME: &str = "cid";

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("Failed to interact with SurrealDB: {0}")]
    GetDbError(anyhow::Error),
    #[error("SurrealDB error: {0}")]
    SurrealDB(#[from] surrealdb::Error),
    #[error("Failed to write data: {0}")]
    WriteError(#[from] std::io::Error),
    #[error("No data found for CID")]
    NotFound,
}

impl DataStore for SurrealDB {
    type Error = DataStoreError;

    async fn delete(&self, cid: &Cid) -> Result<(), Self::Error> {
        let id = Thing::from((
            Table::from(CID_TABLE_NAME).to_string(),
            Id::String(cid.to_string()),
        ));

        self.db.delete::<Option<GetData>>(id).await?;

        Ok(())
    }
    async fn get(&self, cid: &Cid) -> Result<Option<GetDataResults>, Self::Error> {
        let id = Thing::from((
            Table::from(CID_TABLE_NAME).to_string(),
            Id::String(cid.to_string()),
        ));

        let res: GetData = match self.db.select(id).await? {
            Some(res) => res,
            None => return Ok(None),
        };

        Ok(Some(GetDataResults {
            size: res.data.len(),
            data: res.data,
        }))
    }
    async fn put(&self, cid: &Cid, data: Vec<u8>) -> Result<PutDataResults, Self::Error> {
        let db = self.data_db().await.map_err(Self::Error::GetDbError)?;

        let id = Thing::from((
            Table::from(CID_TABLE_NAME).to_string(),
            Id::String(cid.to_string()),
        ));

        let size = data.len();

        db.create::<Option<GetData>>(id)
            .content(CreateData {
                cid: cid.to_string(),
                data,
            })
            .await?;

        Ok(PutDataResults { size })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn store() -> SurrealDB {
        SurrealDB::new().await.expect("Failed to create SurrealDB")
    }

    #[tokio::test]
    async fn test_all_methods() {
        let store = store().await;
        crate::store::tests::data::test_all_methods(store).await;
    }
}
