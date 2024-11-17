use dwn_core::message::Message;
use native_db::{native_db, ToKey};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_model(id = 1, version = 1, with = native_model::rmp_serde_1_3::RmpSerdeNamed)]
#[native_db]
pub struct InitialEntry {
    /// (target, record id)
    #[primary_key]
    pub key: (String, String),
    pub entry: Message,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[native_model(id = 2, version = 1, with = native_model::rmp_serde_1_3::RmpSerdeNamed)]
#[native_db]
pub struct LatestEntry {
    /// (target, record id)
    #[primary_key]
    pub key: (String, String),
    pub entry: Message,
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
