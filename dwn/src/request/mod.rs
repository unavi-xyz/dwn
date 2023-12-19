use serde::{Deserialize, Serialize};

pub mod data;
pub mod message;
pub mod records;

pub use iana_media_types as media_types;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestBody {
    pub messages: Vec<message::Message>,
}
