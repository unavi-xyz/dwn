use serde::{Deserialize, Serialize};

use crate::message::descriptor::{Interface, Method};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ProtocolsQuery {
    interface: Interface,
    method: Method,

    pub filter: ProtocolsFilter,
}

impl ProtocolsQuery {
    pub fn new(protocol: String) -> Self {
        ProtocolsQuery {
            interface: Interface::Protocols,
            method: Method::Query,

            filter: ProtocolsFilter {
                protocol,
                versions: vec![],
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ProtocolsFilter {
    pub protocol: String,
    pub versions: Vec<String>,
}
