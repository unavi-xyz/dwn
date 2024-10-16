use serde::Serialize;

use super::{
    cid::{compute_cid_cbor, CidGenerationError},
    Descriptor,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordIdGeneration {
    descriptor_cid: String,
}

impl Descriptor {
    pub fn compute_record_id(&self) -> Result<String, CidGenerationError> {
        let generator = RecordIdGeneration {
            descriptor_cid: compute_cid_cbor(self)?,
        };
        compute_cid_cbor(&generator)
    }
}
