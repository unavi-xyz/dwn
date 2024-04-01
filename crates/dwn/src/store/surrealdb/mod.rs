//! Data and message store implementations using a SurrealDB database.

use surrealdb::{
    engine::local::{Db, Mem},
    Connection, Surreal,
};

pub mod data;
pub mod message;
mod ql;

pub struct SurrealStore<T: Connection> {
    pub db: Surreal<T>,
    pub namepace: String,
}

impl<T: Connection> From<Surreal<T>> for SurrealStore<T> {
    fn from(db: Surreal<T>) -> Self {
        Self {
            db,
            namepace: "dwn".to_string(),
        }
    }
}

impl<T: Connection> Clone for SurrealStore<T> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            namepace: self.namepace.clone(),
        }
    }
}

impl SurrealStore<Db> {
    /// Creates a new in-memory SurrealDB instance.
    pub async fn new() -> Result<Self, anyhow::Error> {
        let surreal = Surreal::new::<Mem>(()).await?;

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
