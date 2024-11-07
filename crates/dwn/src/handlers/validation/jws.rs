use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use dwn_core::message::Jws;
use jose_jwk::{jose_jwa::Signing, EcCurves, Key};
use ring::signature::{VerificationAlgorithm, ECDSA_P256_SHA256_ASN1, ECDSA_P384_SHA384_ASN1};
use tracing::debug;
use xdid::{
    core::{did::Did, document::VerificationRole},
    resolver::DidResolver,
};

use super::ValidationError;

pub async fn validate_jws(
    did: &Did,
    jws: &Jws,
    role: VerificationRole,
) -> Result<(), ValidationError> {
    // Verify signatures.
    if jws.signatures.is_empty() {
        return Err(ValidationError::MissingSignature);
    }

    let resolver = DidResolver::new()?;

    for signature in jws.signatures.iter() {
        match &signature.header.alg {
            Signing::Es256 => {}
            _ => return Err(ValidationError::UnsupportedAlgorithm),
        }

        // Resolve key URL.
        let document = resolver.resolve(&signature.header.kid.did).await?;

        let Some(vc) = document.resolve_verification_method(&signature.header.kid, role) else {
            debug!(
                "Failed to resolve verification method for kid: {}",
                signature.header.kid
            );
            return Err(ValidationError::InvalidKid);
        };

        if vc.id.did != *did {
            debug!(
                "Verification method id ({}) does not match DID ({})",
                vc.id.did, did
            );
            return Err(ValidationError::InvalidKid);
        }

        // Validate signature.
        let header_str = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_string(&signature.header)?);
        let signed_payload = header_str + "." + &jws.payload;
        let signature = BASE64_URL_SAFE_NO_PAD.decode(&signature.signature)?;

        if let Some(jwk) = &vc.public_key_jwk {
            match &jwk.key {
                Key::Ec(ec) => {
                    let mut public_key = vec![0x04];
                    public_key.extend(ec.x.as_ref());
                    public_key.extend(ec.y.as_ref());

                    match &ec.crv {
                        EcCurves::P256 => {
                            ECDSA_P256_SHA256_ASN1
                                .verify(
                                    public_key.as_slice().into(),
                                    signed_payload.as_bytes().into(),
                                    signature.as_slice().into(),
                                )
                                .map_err(|e| {
                                    debug!("P256 signature verification failed: {:?}", e);
                                    ValidationError::InvalidSignature
                                })?;
                        }
                        EcCurves::P384 => {
                            ECDSA_P384_SHA384_ASN1
                                .verify(
                                    public_key.as_slice().into(),
                                    signed_payload.as_bytes().into(),
                                    signature.as_slice().into(),
                                )
                                .map_err(|e| {
                                    debug!("P384 signature verification failed: {:?}", e);
                                    ValidationError::InvalidSignature
                                })?;
                        }
                        _ => return Err(ValidationError::UnsupportedKey),
                    }
                }
                _ => return Err(ValidationError::UnsupportedKey),
            }
        } else {
            // TODO: support publicKeyMultibase
            return Err(ValidationError::UnsupportedKey);
        }
    }

    Ok(())
}
