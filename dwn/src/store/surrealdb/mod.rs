//! Message store implementation using an embedded SurrealDB database.
//! Saves to the file system when run natively, or to IndexedDB when run in the browser.

use std::sync::Arc;

use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};

pub mod data;
pub mod message;
pub mod model;

const NAMESPACE: &str = "dwn";
const DATA_DB_NAME: &str = "data";
const MESSAGE_DB_NAME: &str = "message";

#[derive(Clone)]
pub struct SurrealDB {
    db: Arc<Surreal<Db>>,
}

impl SurrealDB {
    pub async fn new() -> Result<Self, surrealdb::Error> {
        let db = Surreal::new::<Mem>(()).await?;
        Ok(SurrealDB { db: Arc::new(db) })
    }

    pub async fn data_db(&self) -> Result<Arc<Surreal<Db>>, anyhow::Error> {
        self.db.use_ns(NAMESPACE).use_db(DATA_DB_NAME).await?;
        Ok(self.db.clone())
    }

    pub async fn message_db(&self) -> Result<Arc<Surreal<Db>>, anyhow::Error> {
        self.db.use_ns(NAMESPACE).use_db(MESSAGE_DB_NAME).await?;
        Ok(self.db.clone())
    }
}
