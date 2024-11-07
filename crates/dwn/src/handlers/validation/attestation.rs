use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use dwn_core::message::{cid::compute_cid_cbor, Message};
use xdid::core::{did::Did, document::VerificationRole};

use super::{jws::validate_jws, ValidationError};

pub async fn validate_attestation(did: &Did, msg: &Message) -> Result<(), ValidationError> {
    // Verify payload.
    let cid = compute_cid_cbor(&msg.descriptor)?;

    let attestation = msg
        .attestation
        .as_ref()
        .ok_or(ValidationError::MissingSignature)?;

    if attestation.payload != BASE64_URL_SAFE_NO_PAD.encode(cid) {
        return Err(ValidationError::InvalidPayload);
    }

    // Validate JWS.
    validate_jws(did, attestation, VerificationRole::Assertion).await?;

    Ok(())
}
