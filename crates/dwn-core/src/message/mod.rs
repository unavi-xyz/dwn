use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};

pub use mime;

pub mod cid;
pub mod data;
mod record_id;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct Message {
    pub record_id: String,
    pub data: Option<data::Data>,
    pub descriptor: Descriptor,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct Descriptor {
    pub interface: Interface,
    pub method: Method,
    pub data_cid: Option<String>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub data_format: Option<mime::Mime>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Interface {
    Permissions,
    Protocols,
    Records,
}

impl Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(self).unwrap())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Method {
    Read,
    Query,
    Write,
    Subscribe,
    Delete,
    Configure,
    Request,
    Grant,
    Revocation,
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(self).unwrap())
    }
}
