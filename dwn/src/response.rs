use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResponseBody {
    pub replies: Option<Vec<MessageResult>>,
    pub status: Option<Status>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Status {
    pub code: u16,
    pub detail: Option<String>,
}

impl Status {
    pub fn new(code: u16, detail: Option<&str>) -> Self {
        Self {
            code,
            detail: detail.map(|s| s.to_string()),
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageResult {
    pub entries: Option<Vec<Value>>,
    pub status: Status,
}

impl MessageResult {
    pub fn ok() -> Self {
        Self {
            status: Status::new(200, Some("OK")),
            entries: None,
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            status: Status::new(401, Some("Unauthorized")),
            entries: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            status: Status::new(500, Some(&message)),
            entries: None,
        }
    }
}
