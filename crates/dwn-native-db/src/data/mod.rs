use std::sync::LazyLock;

use native_db::Models;

mod v1;

pub use v1::*;

pub static MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut models = Models::new();
    models.define::<v1::Record>().unwrap();
    models
});