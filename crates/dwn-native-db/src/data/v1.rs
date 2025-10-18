use dwn_core::message::{Message, data::Data, descriptor::ProtocolDefinition};
use native_db::*;
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

use crate::data::VersionKey;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_db]
#[native_model(id = 1, version = 1, with = native_model::rmp_serde_1_3::RmpSerdeNamed)]
pub struct InitialEntry {
    /// (target, record id)
    #[primary_key]
    pub key: (String, String),
    pub entry: Message,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_db]
#[native_model(id = 2, version = 1, with = native_model::rmp_serde_1_3::RmpSerdeNamed)]
pub struct LatestEntry {
    /// (target, record id)
    #[primary_key]
    pub key: (String, String),
    pub entry: Message,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_db]
#[native_model(id = 3, version = 1, with = native_model::rmp_serde_1_3::RmpSerdeNamed)]
pub struct CidData {
    /// (target, cid)
    #[primary_key]
    pub key: (String, String),
    pub data: Option<Data>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[native_db]
#[native_model(id = 4, version = 1, with = native_model::rmp_serde_1_3::RmpSerdeNamed)]
pub struct RefCount {
    /// (target, cid)
    #[primary_key]
    pub key: (String, String),
    pub count: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_db]
#[native_model(id = 5, version = 1, with = native_model::rmp_serde_1_3::RmpSerdeNamed)]
pub struct Protocol {
    /// (target, protocol)
    #[primary_key]
    pub key: (String, String),
    #[secondary_key]
    pub version: VersionKey,
    pub definition: ProtocolDefinition,
}

#[cfg(test)]
mod tests {
    use dwn_core::message::descriptor::RecordsWriteBuilder;

    use super::*;

    #[test]
    fn test_serialize_initial_entry() {
        let msg = RecordsWriteBuilder::default().build().unwrap();

        let val = InitialEntry {
            key: ("did".to_string(), msg.record_id.clone()),
            entry: msg,
        };

        let ser = native_db::bincode_encode_to_vec(&val).unwrap();
        let (des, _) = native_db::bincode_decode_from_slice::<InitialEntry>(&ser).unwrap();
        assert_eq!(des, val);
    }

    #[test]
    fn test_serialize_latest_entry() {
        let msg = RecordsWriteBuilder::default().build().unwrap();

        let val = LatestEntry {
            key: ("did".to_string(), msg.record_id.clone()),
            entry: msg,
        };

        let ser = native_db::bincode_encode_to_vec(&val).unwrap();
        let (des, _) = native_db::bincode_decode_from_slice::<LatestEntry>(&ser).unwrap();
        assert_eq!(des, val);
    }
}
