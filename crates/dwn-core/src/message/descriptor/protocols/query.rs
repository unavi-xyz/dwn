use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::message::descriptor::{Interface, Method};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolsQuery {
    interface: Interface,
    method: Method,
    pub filter: ProtocolFilter,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolFilter {
    pub protocol: Option<String>,
    pub versions: Option<Vec<semver::Version>>,
}
