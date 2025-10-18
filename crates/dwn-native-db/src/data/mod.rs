use std::sync::LazyLock;

use dwn_core::message::Version;
use native_db::{Key, Models, ToKey};

mod v1;

use serde::{Deserialize, Serialize};
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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionKey(pub Version);

impl ToKey for VersionKey {
    fn key_names() -> Vec<String> {
        vec!["VersionKey".to_string()]
    }

    fn to_key(&self) -> Key {
        Key::new(self.0.to_string().into_bytes())
    }
}
