use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use dwn_core::message::{Message, cid::compute_cid_cbor};
use tracing::debug;
use xdid::core::{did::Did, document::VerificationRole};

use super::{ValidationError, jws::validate_jws};

pub async fn validate_attestation(did: &Did, msg: &Message) -> Result<(), ValidationError> {
    // Verify payload.
    let cid = compute_cid_cbor(&msg.descriptor)?;

    let attestation = msg
        .attestation
        .as_ref()
        .ok_or(ValidationError::MissingSignature)?;

    if attestation.payload != BASE64_URL_SAFE_NO_PAD.encode(cid) {
        debug!("Attestation payload does not match base64 encoded CID");
        return Err(ValidationError::InvalidPayload);
    }

    // Validate JWS.
    validate_jws(did, attestation, VerificationRole::Assertion).await?;

    Ok(())
}
