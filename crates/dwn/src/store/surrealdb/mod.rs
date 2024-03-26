//! Data and message store implementations using a SurrealDB database.

use surrealdb::{
    engine::local::{Db, Mem},
    Connection, Surreal,
};

pub mod data;
pub mod message;

const NAMESPACE: &str = "dwn";
const DATA_DB_NAME: &str = "data";
const MESSAGE_DB_NAME: &str = "message";

pub struct SurrealStore<T: Connection>(pub Surreal<T>);

impl<T: Connection> Clone for SurrealStore<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl SurrealStore<Db> {
    /// Creates a new in-memory SurrealDB instance.
    pub async fn new() -> Result<Self, anyhow::Error> {
        let surreal = Surreal::new::<Mem>(()).await?;
        Ok(Self(surreal))
    }
}

impl<T: Connection> SurrealStore<T> {
    pub async fn data_db(&self) -> Result<Surreal<T>, anyhow::Error> {
        self.0.use_ns(NAMESPACE).use_db(DATA_DB_NAME).await?;
        Ok(self.0.clone())
    }

    pub async fn message_db(&self) -> Result<Surreal<T>, anyhow::Error> {
        self.0.use_ns(NAMESPACE).use_db(MESSAGE_DB_NAME).await?;
        Ok(self.0.clone())
    }
}
