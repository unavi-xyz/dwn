use serde::{Deserialize, Serialize};

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
    pub fn new(code: u16, detail: Option<String>) -> Self {
        Self { code, detail }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageResult {
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<Entry>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {}
