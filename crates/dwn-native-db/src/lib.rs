//! DWN backend implementation using [native_db](https://github.com/vincent-herlemont/native_db),
//! a simple embedded database built on [redb](https://github.com/cberner/redb).

use std::{path::Path, sync::Arc};

use native_db::{Builder, Database, db_type};

mod data;
mod data_store;
mod record_store;

#[derive(Clone)]
pub struct NativeDbStore<'a>(Arc<Database<'a>>);

impl NativeDbStore<'_> {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, db_type::Error> {
        let db = Builder::new().create(&data::MODELS, path)?;
        Ok(Self(Arc::new(db)))
    }

    pub fn new_in_memory() -> Result<Self, db_type::Error> {
        let db = Builder::new().create_in_memory(&data::MODELS)?;
        Ok(Self(Arc::new(db)))
    }
}
