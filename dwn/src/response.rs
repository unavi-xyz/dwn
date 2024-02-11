use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResponseBody {
    pub replies: Option<Vec<MessageResult>>,
    pub status: Option<Status>,
}

impl ResponseBody {
    pub fn did_not_found() -> Self {
        Self {
            replies: None,
            status: Some(Status::new(
                404,
                Some("Target DID not found within the Decentralized Web Node"),
            )),
        }
    }
    pub fn error() -> Self {
        Self {
            replies: None,
            status: Some(Status::new(
                500,
                Some("The request could not be processed correctly"),
            )),
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageResult {
    pub entries: Option<Vec<Value>>,
    pub status: Status,
}

pub struct MessageStatus;

impl MessageStatus {
    pub fn ok() -> Status {
        Status::new(200, Some("OK"))
    }
    pub fn bad_request() -> Status {
        Status::new(
            400,
            Some("The message was malformed or improperly constructed"),
        )
    }
    pub fn unauthorized() -> Status {
        Status::new(401, Some("The message failed authorization requirements"))
    }
    pub fn interface_not_implemented() -> Status {
        Status::new(501, Some("The interface method is not implemented"))
    }
    pub fn resource_consumption_limit_exceeded() -> Status {
        Status::new(429, Some("Resource consumption has exceeded tolerances"))
    }
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
