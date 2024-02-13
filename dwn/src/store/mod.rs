#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "s3")]
pub mod s3;
#[cfg(feature = "surrealdb")]
pub mod surrealdb;

pub trait DataStore {}

pub trait MessageStore {}
