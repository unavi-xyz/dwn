use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Body {
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    #[serde(rename = "recordId")]
    pub record_id: String,
    pub data: Option<String>,
    pub descriptor: Descriptor,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Descriptor {
    pub interface: String,
    pub method: String,
    #[serde(rename = "dataCid")]
    pub data_cid: Option<String>,
    #[serde(rename = "dataFormat")]
    pub data_format: Option<String>,
}
