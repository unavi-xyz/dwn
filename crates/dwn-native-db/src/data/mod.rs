use std::sync::LazyLock;

use native_db::Models;

mod v1;

pub use v1::*;

pub static MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut models = Models::new();
    models.define::<v1::InitialEntry>().unwrap();
    models.define::<v1::LatestEntry>().unwrap();
    models.define::<v1::CidData>().unwrap();
    models.define::<v1::RefCount>().unwrap();
    models.define::<v1::Protocol>().unwrap();
    models
});
