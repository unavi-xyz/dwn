use serde::{Deserialize, Serialize};

pub mod data;
pub mod descriptor;
pub mod message;

pub use iana_media_types as media_types;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestBody {
    pub messages: Vec<message::Message>,
}

impl RequestBody {
    pub fn new(messages: Vec<message::Message>) -> Self {
        Self { messages }
    }
}
