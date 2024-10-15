//! DWN backend implementation using `[native_db](https://github.com/vincent-herlemont/native_db)`,
//! a simple embedded database built on `[redb](https://github.com/cberner/redb)`.

use std::path::Path;

use native_db::{db_type, Builder, Database};

mod data;
mod record_store;

pub struct NativeDbStore<'a> {
    db: Database<'a>,
}

impl NativeDbStore<'_> {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, db_type::Error> {
        let db = Builder::new().create(&data::MODELS, path)?;
        Ok(Self { db })
    }

    pub fn new_in_memory() -> Result<Self, db_type::Error> {
        let db = Builder::new().create_in_memory(&data::MODELS)?;
        Ok(Self { db })
    }
}
