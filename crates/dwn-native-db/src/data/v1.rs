use native_db::*;
use native_model::{Model, native_model};
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_db]
#[native_model(id = 1, version = 1)]
pub struct InitialEntry {
    /// (target, record id)
    #[primary_key]
    pub key: (String, String),
    /// Message
    pub entry: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_db]
#[native_model(id = 2, version = 1)]
pub struct LatestEntry {
    /// (target, record id)
    #[primary_key]
    pub key: (String, String),
    /// Message
    pub entry: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_db]
#[native_model(id = 3, version = 1)]
pub struct CidData {
    /// (target, cid)
    #[primary_key]
    pub key: (String, String),
    pub data: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[native_db]
#[native_model(id = 4, version = 1)]
pub struct RefCount {
    /// (target, cid)
    #[primary_key]
    pub key: (String, String),
    pub count: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_db]
#[native_model(id = 5, version = 1)]
pub struct Protocol {
    /// (target, protocol)
    #[primary_key]
    pub key: (String, String),
    pub version: Version,
    /// Serialized [ProtocolDefinition].
    /// Cannot be stored directly because of non-deteministic maps.
    pub definition: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use dwn_core::message::descriptor::RecordsWriteBuilder;

    use super::*;

    #[test]
    fn test_serialize_initial_entry() {
        let msg = RecordsWriteBuilder::default().build().unwrap();
        let msg_b = serde_json::to_vec(&msg).unwrap();

        let val = InitialEntry {
            key: ("did".to_string(), msg.record_id.clone()),
            entry: msg_b,
        };

        let ser = native_db::bincode_encode_to_vec(&val).unwrap();
        let (des, _) = native_db::bincode_decode_from_slice::<InitialEntry>(&ser).unwrap();

        assert_eq!(des, val);
    }

    #[test]
    fn test_serialize_latest_entry() {
        let msg = RecordsWriteBuilder::default().build().unwrap();
        let msg_b = serde_json::to_vec(&msg).unwrap();

        let val = LatestEntry {
            key: ("did".to_string(), msg.record_id.clone()),
            entry: msg_b,
        };

        let ser = native_db::bincode_encode_to_vec(&val).unwrap();
        let (des, _) = native_db::bincode_decode_from_slice::<LatestEntry>(&ser).unwrap();
        assert_eq!(des, val);
    }
}
