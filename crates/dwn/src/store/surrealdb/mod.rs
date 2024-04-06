//! Data and message store implementations using a SurrealDB database.

use surrealdb::{Connection, Surreal};

pub mod data;
pub mod message;
mod ql;

pub struct SurrealStore<T: Connection> {
    pub db: Surreal<T>,
    pub namepace: String,
}

impl<C: Connection> Clone for SurrealStore<C> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            namepace: self.namepace.clone(),
        }
    }
}

impl<C: Connection> SurrealStore<C> {
    pub async fn new(surreal: Surreal<C>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            db: surreal,
            namepace: "dwn".to_string(),
        })
    }
}

impl<T: Connection> SurrealStore<T> {
    pub async fn data_db(&self) -> Result<Surreal<T>, anyhow::Error> {
        let db = self.db.clone();
        db.use_ns(&self.namepace).use_db("data").await?;
        Ok(db)
    }

    pub async fn message_db(&self, tenant: &str) -> Result<Surreal<T>, anyhow::Error> {
        let db = self.db.clone();
        db.use_ns(&self.namepace).use_db(tenant).await?;
        Ok(db)
    }
}
