use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use dwn_core::message::{AuthPayload, Message, cid::compute_cid_cbor};
use tracing::debug;
use xdid::core::{did::Did, document::VerificationRole};

use super::{ValidationError, jws::validate_jws};

pub async fn validate_authorization(did: &Did, msg: &Message) -> Result<(), ValidationError> {
    // Verify payload.
    let authorization = msg
        .authorization
        .as_ref()
        .ok_or(ValidationError::MissingSignature)?;

    let decoded = BASE64_URL_SAFE_NO_PAD.decode(&authorization.payload)?;
    let payload = serde_json::from_slice::<AuthPayload>(&decoded)?;

    let cid = compute_cid_cbor(&msg.descriptor)?;

    if payload.descriptor_cid != cid {
        debug!(
            "Descriptor cid ({}) does not match computed cid ({})",
            payload.descriptor_cid, cid
        );
        return Err(ValidationError::InvalidPayload);
    }

    if let Some(attestation_cid) = &payload.attestation_cid {
        let Some(found) = msg.attestation.as_ref().map(|j| &j.payload) else {
            debug!("No attestation found");
            return Err(ValidationError::InvalidPayload);
        };

        if attestation_cid != found {
            debug!("Attestation CID does not match found attestation");
            return Err(ValidationError::InvalidPayload);
        }
    }

    // Validate JWS.
    validate_jws(did, authorization, VerificationRole::Authentication).await?;

    Ok(())
}
