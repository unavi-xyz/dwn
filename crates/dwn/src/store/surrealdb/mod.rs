//! Data and message store implementations using a SurrealDB database.

use surrealdb::{Connection, Surreal};

pub mod data;
pub mod message;
mod ql;

pub struct SurrealStore<C: Connection> {
    pub db: Surreal<C>,
    pub namespace: String,
}

impl<C: Connection> Clone for SurrealStore<C> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            namespace: self.namespace.clone(),
        }
    }
}

impl<C: Connection> SurrealStore<C> {
    pub async fn new(surreal: Surreal<C>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            db: surreal,
            namespace: "dwn".to_string(),
        })
    }
}

impl<T: Connection> SurrealStore<T> {
    pub async fn data_db(&self) -> Result<Surreal<T>, anyhow::Error> {
        let db = self.db.clone();
        db.use_ns(&self.namespace).use_db("data").await?;
        Ok(db)
    }

    pub async fn message_db(&self, tenant: &str) -> Result<Surreal<T>, anyhow::Error> {
        let db = self.db.clone();
        db.use_ns(&self.namespace).use_db(tenant).await?;
        Ok(db)
    }
}
