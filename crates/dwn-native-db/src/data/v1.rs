use native_db::{native_db, ToKey};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct Record {
    #[primary_key]
    pub id: String,
}
