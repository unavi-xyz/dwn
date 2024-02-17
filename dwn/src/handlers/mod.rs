use std::error::Error;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::message::Message;

pub mod records;

pub trait MethodHandler {
    type Error: Error + Send + Sync + 'static;

    fn handle(&self, tenant: &str, message: Message) -> Result<MessageReply, Self::Error>;
}

pub struct MessageReply {
    pub status: Status,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Status {
    pub code: u16,
    pub detail: String,
}

impl Status {
    fn ok() -> Self {
        Status {
            code: 200,
            detail: "OK".to_string(),
        }
    }
}
