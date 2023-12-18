use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResponseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<MessageResult>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Status {
    pub code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageResult {
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<Value>>,
}

impl MessageResult {
    pub fn new(entries: Vec<Value>) -> Self {
        Self {
            status: Status::new(200, Some("OK")),
            entries: Some(entries),
        }
    }
}
